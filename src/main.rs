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

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file } => {
            println!("Reading: {file}...");
            let content =
                fs::read_to_string(file).with_context(|| format!("Failed to read file: {file}"))?;

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
            let path = PathBuf::from(file);
            let canonical_path = fs::canonicalize(&path)
                .with_context(|| format!("Failed to resolve path: {file}"))?;

            println!("Reading: {}...", canonical_path.display());
            let content =
                fs::read_to_string(&canonical_path).with_context(|| format!("Failed to read file: {file}"))?;

            let steps = core::parser::parse_readme(&content);

            if steps.is_empty() {
                println!("No sections (headers) found in the Markdown file.");
                return Ok(());
            }

            println!("Launching UI for {} steps...", steps.len());
            ui::run_tui(steps, canonical_path)?;
        }
        Commands::Check { file } => {
            println!("Checking dependencies for: {file}...");
            let content =
                fs::read_to_string(file).with_context(|| format!("Failed to read file: {file}"))?;

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
