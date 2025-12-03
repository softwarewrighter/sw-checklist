//! CLI argument definitions

use clap::Parser;
use std::path::PathBuf;

/// CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the project to check (defaults to current directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,
}

/// Parse command line arguments
pub fn parse() -> Cli {
    Cli::parse()
}
