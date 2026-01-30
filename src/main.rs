use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "compass")]
#[command(about = "🧭 Compass: Interactive README Navigator", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse and display the structure of a README
    Parse { file: String },
    /// Check if system dependencies are met
    Check { file: String },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { file } => {
            println!("Reading: {}...", file);
            // TODO: Call parser::parse_readme
            println!("Parser logic coming soon!");
        }
        Commands::Check { file } => {
            println!("Checking dependencies for: {}...", file);
            // TODO: Call executor::check_deps
        }
    }
}
