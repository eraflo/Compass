// Copyright 2026 eraflo
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod core;
mod ui;

use anyhow::Context;
use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "compass")]
#[command(about = "🧭 Compass: Interactive README Navigator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable sandbox mode using Docker
    #[arg(short, long, global = true)]
    sandbox: bool,

    /// Docker image to use in sandbox mode
    #[arg(long, global = true, default_value = "ubuntu:latest")]
    image: String,

    /// Run in Headless mode (JSON-RPC over Stdio)
    #[arg(long, global = true)]
    headless: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and display a summary of the README
    Parse { file: String },
    /// Launch the interactive TUI
    Tui {
        file: String,
        /// Share this session with others (Host mode)
        #[arg(long)]
        share: bool,
    },
    /// Check if system dependencies are met
    Check { file: String },
    /// Join a shared session (Guest mode)
    Join {
        /// The secure connection URL (wss://.../?pin=...)
        url: String,
    },
    /// Search for community runbooks
    Search {
        /// Keywords to search for
        query: String,
    },
    /// Scan current directory for Markdown files
    Scan {
        /// Path to scan (default: current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
    /// Clone a runbook from the registry
    Clone {
        /// Name of the runbook (from registry)
        name: String,
        /// Destination filename (optional)
        destination: Option<String>,
    },
}

async fn load_readme(file: &str) -> anyhow::Result<(String, PathBuf, bool)> {
    if file.starts_with("http://") || file.starts_with("https://") {
        println!("Downloading remote README from {}...", file);
        let content = core::fetcher::fetch_remote_content(file)?;
        Ok((content, PathBuf::from(file), true))
    } else {
        let path = PathBuf::from(file);
        if path.exists() {
            let canonical_path = fs::canonicalize(&path)?;
            println!("Reading: {}...", canonical_path.display());
            let content = fs::read_to_string(&canonical_path)
                .with_context(|| format!("Failed to read file: {file}"))?;
            Ok((content, canonical_path, false))
        } else {
            // Try matching registry
            println!(
                "File not found locally. Searching registry for '{}'...",
                file
            );
            match core::ecosystem::hub::resolve_runbook(file).await {
                Ok(Some(runbook)) => {
                    println!(
                        "Found '{}' in registry. Downloading from: {}",
                        runbook.name, runbook.url
                    );
                    let content = core::fetcher::fetch_remote_content(&runbook.url)?;
                    Ok((content, PathBuf::from(runbook.url), true))
                }
                _ => {
                    anyhow::bail!("File '{}' not found locally or in registry.", file);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize Rustls Crypto Provider (Ring)
    // This is required for Rustls 0.23+ to function correctly without panicking.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file } => {
            let (content, _, _) = load_readme(file).await?;
            let (steps, hooks) = core::parser::parse_readme(&content);

            if hooks.is_some() {
                println!("🪝 Hooks detected: Yes");
            }
            println!("Detected {} steps:", steps.len());
            for (i, step) in steps.iter().enumerate() {
                println!(
                    "  {}. {} ({} code blocks)",
                    i + 1,
                    step.title,
                    step.code_blocks.len()
                );
            }
        }
        Commands::Tui { file, share } => {
            // Check for sandbox availability if enabled
            if cli.sandbox {
                if let Err(e) = core::infrastructure::docker::ensure_docker_available() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                println!("📦 Sandbox mode enabled (Image: {})", cli.image);
            }

            let (content, path, is_remote) = load_readme(file).await?;
            let (steps, hooks) = core::parser::parse_readme(&content);

            if steps.is_empty() {
                println!("No sections (headers) found in the Markdown file.");
                return Ok(());
            }

            // Trigger Pre-run hook (environment setup)
            let mut hooks_trusted = false;

            if let Some(h) = hooks.as_ref()
                && h.has_any()
            {
                if !cli.headless {
                    println!("\n⚠️  SECURITY WARNING ⚠️");
                    println!(
                        "This runbook contains automation hooks (pre_run, post_run, on_failure, etc.)."
                    );
                    if let Some(cmd) = &h.pre_run {
                        println!("It wants to execute this command IMMEDIATELY:");
                        println!("  Command: {}", cmd);
                    }
                    println!("Compass cannot verify if these commands are safe.");
                    println!("Do you trust this runbook? [y/N]");

                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().eq_ignore_ascii_case("y") {
                        hooks_trusted = true;
                        // Trigger pre_run immediately if trusted
                        core::ecosystem::hooks::trigger_hook(
                            &h.pre_run,
                            &std::collections::HashMap::new(),
                        );
                    } else {
                        println!("❌ Hooks disabled for this session.");
                    }
                } else {
                    // Headless always trusts (assumes automation environment)
                    hooks_trusted = true;
                    eprintln!("[HEADLESS] Executing pre-run hook...");
                    core::ecosystem::hooks::trigger_hook(
                        &h.pre_run,
                        &std::collections::HashMap::new(),
                    );
                }
            }

            // Headless Mode Check
            if cli.headless {
                println!("Running in HEADLESS mode (JSON-RPC)...");
                core::ecosystem::rpc::start_headless_server(
                    steps,
                    path,
                    cli.sandbox,
                    cli.image.clone(),
                )
                .await?;
                return Ok(());
            }

            // Collaboration Setup
            let mut collab_session = None;

            if *share {
                // Generate Certs & PIN *before* spawning server/TUI
                println!("Generating Secure Certificate (TLS 1.3)...");
                let (certs, key, pin) = core::collab::security::generate_self_signed()?;

                let ip = local_ip_address::local_ip()
                    .unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST));
                let secure_link = format!("wss://{}:3030/?pin={}", ip, pin);

                println!("\n🔐 Public Secure Session Ready!");
                println!("👉  JOIN LINK:  {}", secure_link);
                println!("    (Share this link securely. It acts as both key and certificate.)\n");

                println!("Press ENTER to launch the Host Interface...");
                let mut input = String::new();
                std::io::stdin().read_line(&mut input)?;

                let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

                // Spawn the Host Server
                tokio::spawn(async move {
                    if let Err(e) =
                        core::collab::server::start_host_server(rx, certs, key, pin).await
                    {
                        eprintln!("Host server error: {}", e);
                    }
                });

                collab_session = Some(core::collab::session::CollabSession::new(
                    true, // is_host
                    Some(secure_link),
                    Some(tx), // App writes to this
                    None,     // Host doesn't read from guest yet
                ));
            } else {
                println!("Launching UI for {} steps...", steps.len());
            }

            ui::run_tui(
                steps,
                path,
                is_remote,
                cli.sandbox,
                cli.image,
                collab_session,
                hooks,
                hooks_trusted,
            )?;
        }
        Commands::Check { file } => {
            let (content, _, _) = load_readme(file).await?;
            let (steps, _) = core::parser::parse_readme(&content);
            let result = core::executor::check_dependencies(&steps);

            if !result.present.is_empty() {
                println!("\n✅ Present:");
                for cmd in &result.present {
                    println!("   - {cmd}");
                }
            }

            if result.missing.is_empty() {
                println!("\nAll detected dependencies seem to be present!");
            } else {
                println!("\n❌ Missing:");
                for cmd in &result.missing {
                    println!("   - {cmd}");
                }
                println!("\nSome dependencies are missing. Please install them before proceeding.");
            }
        }
        Commands::Join { url } => {
            // Fix URL format if needed
            let url = if url.contains("://") {
                url.clone()
            } else {
                // Default to wss:// for secure default
                format!("wss://{}", url)
            };

            let (tx, rx) = std::sync::mpsc::channel();

            println!("Connecting to {}...", url);

            // Spawn client
            let tx_clone = tx.clone();
            let url_for_client = url.clone();
            tokio::spawn(async move {
                if let Err(e) =
                    core::collab::client::start_guest_client(url_for_client, tx_clone).await
                {
                    eprintln!("Guest client error: {}", e);
                    std::process::exit(1);
                }
            });

            // Wait for Snapshot
            println!("Waiting for session data...");
            let steps = match rx.recv() {
                Ok(core::collab::events::CompassEvent::Snapshot { steps, .. }) => steps,
                Ok(_) => {
                    eprintln!("Error: Expected Snapshot as first message.");
                    std::process::exit(1);
                }
                Err(_) => {
                    eprintln!("Connection closed.");
                    std::process::exit(1);
                }
            };

            let path = PathBuf::from("REMOTE_SESSION");

            let collab_session = Some(core::collab::session::CollabSession::new(
                false,
                Some(url.clone()),
                None,
                Some(rx),
            ));

            println!("Joining session with {} steps...", steps.len());
            ui::run_tui(
                steps,
                path,
                true,
                false,
                "ubuntu:latest".to_string(),
                collab_session,
                None,
                false, // Hooks not trusted/present in guest mode locally
            )?;
        }
        Commands::Search { query } => {
            println!("🔍 Searching Compass Hub for '{}'...", query);
            let results = core::ecosystem::hub::search_remote(query).await?;
            if results.is_empty() {
                println!("No results found.");
            } else {
                println!("Found {} runbooks:", results.len());
                for (i, r) in results.iter().enumerate() {
                    println!("{}. {} ({}) - ⭐ {}", i + 1, r.name, r.author, r.stars);
                    println!("   {} - 🔗 {}", r.description, r.url);
                }
            }
        }
        Commands::Scan { path } => {
            println!("📂 Scanning for runbooks in: {:?}", path);
            let files = core::ecosystem::discovery::scan_directory(path)?;
            if files.is_empty() {
                println!("No runbooks found.");
            } else {
                println!("Found {} local runbooks:", files.len());
                for file in files {
                    println!(" - {}", file.display());
                }
            }
        }
        Commands::Clone { name, destination } => {
            let (url, default_name) = if name.starts_with("http") {
                println!("Downloading from URL...");
                (name.clone(), "runbook.md".to_string())
            } else {
                println!("Searching registry for '{}'...", name);
                if let Some(runbook) = core::ecosystem::hub::resolve_runbook(name).await? {
                    (runbook.url, format!("{}.md", runbook.name))
                } else {
                    anyhow::bail!("Runbook '{}' not found in registry.", name);
                }
            };

            println!("Fetching content from {}...", url);
            let content = core::fetcher::fetch_remote_content(&url)?;
            let filename = destination.as_deref().unwrap_or(&default_name);
            std::fs::write(filename, content)?;
            println!("✅ Successfully cloned into '{}'", filename);
        }
    }

    Ok(())
}
