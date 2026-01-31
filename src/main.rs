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
    Tui { file: String },
    /// Check if system dependencies are met
    Check { file: String },
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

fn main() -> anyhow::Result<()> {
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
        Commands::Tui { file } => {
            if cli.sandbox {
                // Use the core docker module to check availability
                if let Err(e) = core::docker::ensure_docker_available() {
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

            println!("Launching UI for {} steps...", steps.len());
            ui::run_tui(steps, path, is_remote, cli.sandbox, cli.image)?;
        }
        Commands::Check { file } => {
            let (content, _, _) = load_readme(file)?;
            let steps = core::parser::parse_readme(&content);
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
    }

    Ok(())
}
