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
}

fn load_readme(file: &str) -> anyhow::Result<(String, PathBuf, bool)> {
    if file.starts_with("http://") || file.starts_with("https://") {
        println!("Downloading remote README from {}...", file);
        let content = core::fetcher::fetch_remote_content(file)?;
        Ok((content, PathBuf::from(file), true))
    } else {
        let path = PathBuf::from(file);
        let canonical_path = if path.exists() {
            fs::canonicalize(&path)?
        } else {
            path
        };
        println!("Reading: {}...", canonical_path.display());
        let content = fs::read_to_string(&canonical_path)
            .with_context(|| format!("Failed to read file: {file}"))?;
        Ok((content, canonical_path, false))
    }
}

use tokio::runtime::Runtime;

fn main() -> anyhow::Result<()> {
    // Initialize Rustls Crypto Provider (Ring)
    // This is required for Rustls 0.23+ to function correctly without panicking.
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file } => {
            let (content, _, _) = load_readme(file)?;
            let steps = core::parser::parse_readme(&content);

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
            // ... (sandbox check omitted for brevity in thought, but kept in code)
            if cli.sandbox {
                if let Err(e) = core::infrastructure::docker::ensure_docker_available() {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
                println!("📦 Sandbox mode enabled (Image: {})", cli.image);
            }

            let (content, path, is_remote) = load_readme(file)?;
            let steps = core::parser::parse_readme(&content);

            if steps.is_empty() {
                println!("No sections (headers) found in the Markdown file.");
                return Ok(());
            }

            // Collaboration Setup
            let mut collab_session = None;
            let rt = Runtime::new()?;

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
                rt.spawn(async move {
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
            )?;
        }
        Commands::Check { file } => {
            let (content, _, _) = load_readme(file)?;
            let steps = core::parser::parse_readme(&content);
            let result = core::executor::check_dependencies(&steps);
            // ... (keep existing)
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
            let rt = Runtime::new()?;
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
            rt.spawn(async move {
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
            )?;
        }
    }

    Ok(())
}
