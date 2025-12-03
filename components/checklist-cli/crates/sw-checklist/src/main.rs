//! sw-checklist - CLI tool for validating Software Wrighter LLC project conformance

use anyhow::Result;
use checklist_config::ConfigBuilder;
use clap::Parser;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_COMMIT: &str = env!("BUILD_COMMIT_SHA");
const BUILD_TIME: &str = env!("BUILD_TIMESTAMP");
const BUILD_HOST: &str = env!("BUILD_HOST");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

const LONG_VERSION: &str = const_format::formatcp!(
    "{}\n\nCopyright (c) 2025 Michael A Wright\nMIT License\n\nRepository: {}\nBuild Host: {}\nBuild Commit: {}\nBuild Time: {}",
    VERSION,
    REPO,
    BUILD_HOST,
    BUILD_COMMIT,
    BUILD_TIME
);

const AI_INSTRUCTIONS: &str = r#"AI CODING AGENT INSTRUCTIONS:

This tool validates that projects conform to Software Wrighter LLC standards.

USAGE FOR AI AGENTS:
  1. Run this tool on any project to get a checklist of requirements
  2. Address each issue reported by the tool
  3. Re-run to verify all checks pass

CHECKS PERFORMED:
  - Rust edition must be 2024
  - Functions: warns if >25 LOC, fails if >50 LOC
  - Modules: warns if >4 functions, fails if >7 functions
  - Crates: warns if >4 modules, fails if >7 modules
  - CLI binaries: help/version output validation
  - Web UI: favicon, index.html, footer requirements
"#;

/// CLI tool for validating Software Wrighter LLC project conformance
#[derive(Parser)]
#[command(name = "sw-checklist")]
#[command(long_version = LONG_VERSION)]
#[command(about = "CLI tool for validating Software Wrighter LLC project conformance")]
#[command(after_long_help = AI_INSTRUCTIONS)]
struct Cli {
    /// Project path to check (defaults to current directory)
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = ConfigBuilder::new()
        .project_path(cli.path)
        .verbose(cli.verbose)
        .build();

    let exit_code = cli_runner::run(&config)?;
    std::process::exit(exit_code);
}
