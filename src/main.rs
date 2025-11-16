use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const BUILD_COMMIT: &str = env!("BUILD_COMMIT_SHA");
const BUILD_TIME: &str = env!("BUILD_TIMESTAMP");
const BUILD_HOST_NAME: &str = env!("BUILD_HOST");
const REPO: &str = env!("CARGO_PKG_REPOSITORY");

const LONG_VERSION: &str = const_format::formatcp!(
    "{}\n\nCopyright (c) 2025 Michael A Wright\nMIT License\n\nRepository: {}\nBuild Host: {}\nBuild Commit: {}\nBuild Time: {}",
    VERSION, REPO, BUILD_HOST_NAME, BUILD_COMMIT, BUILD_TIME
);

/// CLI tool for validating Software Wrighter LLC project conformance
#[derive(Parser)]
#[command(name = "sw-checklist")]
#[command(long_version = LONG_VERSION)]
#[command(
    about = "CLI tool for validating Software Wrighter LLC project conformance\n\nUse --help for additional details including AI Coding Agent instructions."
)]
#[command(
    long_about = "CLI tool for validating Software Wrighter LLC project conformance\n\n\
    This tool inspects a project directory and checks for compliance with\n\
    Software Wrighter LLC standards and best practices. It automatically\n\
    detects project types (Rust, etc.) and runs appropriate validation checks.\n\n\
    For Rust projects with clap, it validates:\n\
    - Help output (-h vs --help)\n\
    - Version output (-V vs --version)\n\
    - Required metadata in version output\n\
    - AI Coding Agent instructions in help"
)]
#[command(after_long_help = AI_AGENT_INSTRUCTIONS)]
struct Cli {
    /// Project path to check (defaults to current directory)
    #[arg(default_value = ".")]
    project_path: PathBuf,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

const AI_AGENT_INSTRUCTIONS: &str = r#"AI CODING AGENT INSTRUCTIONS:

This tool validates that projects conform to Software Wrighter LLC standards.
It performs various checks based on the project type detected.

USAGE FOR AI AGENTS:
  1. Run this tool on any project to get a checklist of requirements
  2. Address each issue reported by the tool
  3. Re-run to verify all checks pass

EXAMPLE WORKFLOW:
  $ sw-checklist /path/to/project
  # Review output and fix issues
  $ sw-checklist /path/to/project
  # Verify all checks pass

CURRENT CHECKS:
  - Rust projects with clap: Validates help and version output
  - More checks coming soon

For more information, see the repository:
https://github.com/softwarewrighter/sw-checklist
"#;

#[derive(Debug, Clone, Copy, PartialEq)]
enum ProjectType {
    Rust,
    Unknown,
}

#[derive(Debug)]
struct CheckResult {
    name: String,
    passed: bool,
    message: String,
}

impl CheckResult {
    fn pass(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: true,
            message: message.into(),
        }
    }

    fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            passed: false,
            message: message.into(),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let project_path = fs::canonicalize(&cli.project_path)
        .with_context(|| format!("Failed to access project path: {:?}", cli.project_path))?;

    println!("Checking project: {}", project_path.display());
    println!();

    let project_type = detect_project_type(&project_path);
    println!("Project type: {:?}", project_type);
    println!();

    let results = run_checks(&project_path, project_type, cli.verbose)?;

    print_results(&results);

    let passed = results.iter().filter(|r| r.passed).count();
    let failed = results.iter().filter(|r| !r.passed).count();

    println!();
    println!("Summary: {} passed, {} failed", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn detect_project_type(path: &Path) -> ProjectType {
    if path.join("Cargo.toml").exists() {
        ProjectType::Rust
    } else {
        ProjectType::Unknown
    }
}

fn run_checks(path: &Path, project_type: ProjectType, verbose: bool) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    match project_type {
        ProjectType::Rust => {
            results.extend(check_rust_project(path, verbose)?);
        }
        ProjectType::Unknown => {
            results.push(CheckResult::fail(
                "Project Type",
                "Unknown project type - no checks available",
            ));
        }
    }

    Ok(results)
}

fn check_rust_project(path: &Path, verbose: bool) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Check if project uses clap
    let cargo_toml_path = path.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read Cargo.toml at {:?}", cargo_toml_path))?;

    if cargo_toml.contains("clap") {
        results.push(CheckResult::pass(
            "Clap Dependency",
            "Found clap dependency",
        ));

        // Try to find and check the binary
        if let Some(binary_results) = check_rust_binary(path, verbose) {
            results.extend(binary_results);
        } else {
            results.push(CheckResult::fail(
                "Binary Check",
                "Could not find built binary. Run 'cargo build --release' first.",
            ));
        }
    } else {
        results.push(CheckResult::pass(
            "Clap Check",
            "No clap dependency - skipping CLI checks",
        ));
    }

    Ok(results)
}

fn check_rust_binary(path: &Path, verbose: bool) -> Option<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Try to find binary in target/release or target/debug
    let binary_name = get_rust_binary_name(path)?;

    let release_binary = path.join("target/release").join(&binary_name);
    let debug_binary = path.join("target/debug").join(&binary_name);

    let binary_path = if release_binary.exists() {
        release_binary
    } else if debug_binary.exists() {
        debug_binary
    } else {
        return None;
    };

    if verbose {
        println!("  Checking binary: {}", binary_path.display());
    }

    // Check -h vs --help
    results.extend(check_help_flags(&binary_path, verbose));

    // Check -V vs --version
    results.extend(check_version_flags(&binary_path, verbose));

    Some(results)
}

fn get_rust_binary_name(path: &Path) -> Option<String> {
    let cargo_toml = fs::read_to_string(path.join("Cargo.toml")).ok()?;
    let cargo: toml::Value = toml::from_str(&cargo_toml).ok()?;

    // Try [[bin]] section first
    if let Some(bins) = cargo.get("bin").and_then(|b| b.as_array()) {
        if let Some(first_bin) = bins.first() {
            if let Some(name) = first_bin.get("name").and_then(|n| n.as_str()) {
                return Some(name.to_string());
            }
        }
    }

    // Fall back to package name
    cargo
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(|s| s.to_string())
}

fn check_help_flags(binary: &Path, verbose: bool) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let short_help = run_command(binary, &["-h"]);
    let long_help = run_command(binary, &["--help"]);

    match (short_help, long_help) {
        (Ok(short), Ok(long)) => {
            if verbose {
                println!("  -h output ({} bytes)", short.len());
                println!("  --help output ({} bytes)", long.len());
            }

            // Check that --help is longer
            if long.len() > short.len() {
                results.push(CheckResult::pass(
                    "Help Length",
                    format!(
                        "--help ({} bytes) is longer than -h ({} bytes)",
                        long.len(),
                        short.len()
                    ),
                ));
            } else {
                results.push(CheckResult::fail(
                    "Help Length",
                    format!(
                        "--help ({} bytes) should be longer than -h ({} bytes)",
                        long.len(),
                        short.len()
                    ),
                ));
            }

            // Check that --help contains "AI CODING AGENT" or similar
            if long.contains("AI CODING AGENT") || long.contains("AI Coding Agent") {
                results.push(CheckResult::pass(
                    "AI Agent Instructions",
                    "Found AI Coding Agent section in --help",
                ));
            } else {
                results.push(CheckResult::fail(
                    "AI Agent Instructions",
                    "--help should include an 'AI CODING AGENT INSTRUCTIONS' section",
                ));
            }
        }
        (Err(e), _) => {
            results.push(CheckResult::fail(
                "Help -h",
                format!("Failed to run -h: {}", e),
            ));
        }
        (_, Err(e)) => {
            results.push(CheckResult::fail(
                "Help --help",
                format!("Failed to run --help: {}", e),
            ));
        }
    }

    results
}

fn check_version_flags(binary: &Path, verbose: bool) -> Vec<CheckResult> {
    let mut results = Vec::new();

    let short_version = run_command(binary, &["-V"]);
    let long_version = run_command(binary, &["--version"]);

    match (short_version, long_version) {
        (Ok(short), Ok(long)) => {
            if verbose {
                println!("  -V output: {}", short.trim());
                println!("  --version output: {}", long.trim());
            }

            // Check that outputs are identical
            if short == long {
                results.push(CheckResult::pass(
                    "Version Consistency",
                    "-V and --version produce identical output",
                ));
            } else {
                results.push(CheckResult::fail(
                    "Version Consistency",
                    "-V and --version should produce identical output",
                ));
            }

            // Check for required fields in version output
            let version_output = &long;
            let required_fields = [
                ("Copyright", "Copyright (c)"),
                ("License", "MIT License"),
                ("Repository", "https://github.com"),
                ("Build Host", "Build Host:"),
                ("Build Commit", "Build Commit:"),
                ("Build Time", "Build Time:"),
            ];

            for (field_name, pattern) in required_fields {
                if version_output.contains(pattern) {
                    results.push(CheckResult::pass(
                        format!("Version Field: {}", field_name),
                        format!("Found {} in version output", field_name),
                    ));
                } else {
                    results.push(CheckResult::fail(
                        format!("Version Field: {}", field_name),
                        format!("Version output should contain {}", field_name),
                    ));
                }
            }
        }
        (Err(e), _) => {
            results.push(CheckResult::fail(
                "Version -V",
                format!("Failed to run -V: {}", e),
            ));
        }
        (_, Err(e)) => {
            results.push(CheckResult::fail(
                "Version --version",
                format!("Failed to run --version: {}", e),
            ));
        }
    }

    results
}

fn run_command(binary: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(binary)
        .args(args)
        .output()
        .with_context(|| format!("Failed to execute {:?} {:?}", binary, args))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn print_results(results: &[CheckResult]) {
    println!("Check Results:");
    println!("{}", "=".repeat(80));

    for result in results {
        let status = if result.passed {
            "✓ PASS"
        } else {
            "✗ FAIL"
        };
        println!("{} | {}", status, result.name);
        println!("       {}", result.message);
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_rust_project() {
        use std::fs;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let cargo_toml = temp.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"\n").unwrap();

        assert_eq!(detect_project_type(temp.path()), ProjectType::Rust);
    }

    #[test]
    fn test_detect_unknown_project() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        assert_eq!(detect_project_type(temp.path()), ProjectType::Unknown);
    }

    #[test]
    fn test_check_result_creation() {
        let pass = CheckResult::pass("Test", "This passed");
        assert!(pass.passed);
        assert_eq!(pass.name, "Test");
        assert_eq!(pass.message, "This passed");

        let fail = CheckResult::fail("Test", "This failed");
        assert!(!fail.passed);
        assert_eq!(fail.name, "Test");
        assert_eq!(fail.message, "This failed");
    }
}
