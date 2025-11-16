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

    // Find all Cargo.toml files in the project
    let cargo_tomls = find_cargo_tomls(&project_path);

    if cargo_tomls.is_empty() {
        println!("Project type: Unknown");
        println!();
        print_results(&[CheckResult::fail(
            "Project Type",
            "No Cargo.toml files found - no checks available",
        )]);
        println!();
        println!("Summary: 0 passed, 1 failed");
        std::process::exit(1);
    }

    println!("Project type: Rust");
    println!("Found {} Cargo.toml file(s)", cargo_tomls.len());
    println!();

    let results = run_checks(&project_path, &cargo_tomls, cli.verbose)?;

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

fn find_cargo_tomls(path: &Path) -> Vec<PathBuf> {
    use walkdir::WalkDir;

    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml")
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn run_checks(
    project_root: &Path,
    cargo_tomls: &[PathBuf],
    verbose: bool,
) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    for cargo_toml_path in cargo_tomls {
        if verbose {
            println!("Checking: {}", cargo_toml_path.display());
        }

        let cargo_toml = fs::read_to_string(cargo_toml_path)
            .with_context(|| format!("Failed to read Cargo.toml at {:?}", cargo_toml_path))?;

        // Only check crates that use clap
        if !cargo_toml.contains("clap") {
            continue;
        }

        let crate_dir = cargo_toml_path.parent().unwrap();
        results.extend(check_rust_crate(project_root, crate_dir, verbose)?);
    }

    if results.is_empty() {
        results.push(CheckResult::pass(
            "Clap Check",
            "No crates using clap found - skipping CLI checks",
        ));
    }

    Ok(results)
}

fn check_rust_crate(
    project_root: &Path,
    crate_dir: &Path,
    verbose: bool,
) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    let cargo_toml_path = crate_dir.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path)?;
    let cargo: toml::Value = toml::from_str(&cargo_toml)
        .with_context(|| format!("Failed to parse Cargo.toml at {:?}", cargo_toml_path))?;

    let crate_name = cargo
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    results.push(CheckResult::pass(
        format!("Clap Dependency [{}]", crate_name),
        format!("Found clap dependency in {}", crate_name),
    ));

    // Try to find and check binaries for this crate
    if let Some(binary_results) = check_crate_binaries(project_root, &cargo, crate_name, verbose) {
        results.extend(binary_results);
    } else {
        results.push(CheckResult::fail(
            format!("Binary Check [{}]", crate_name),
            format!(
                "Could not find built binaries for {}. Run 'cargo build --release' first.",
                crate_name
            ),
        ));
    }

    Ok(results)
}

fn check_crate_binaries(
    project_root: &Path,
    cargo: &toml::Value,
    crate_name: &str,
    verbose: bool,
) -> Option<Vec<CheckResult>> {
    let mut results = Vec::new();
    let mut found_any_binary = false;

    // Get list of binary names from [[bin]] sections or default to package name
    let binary_names = get_binary_names(cargo);

    for binary_name in binary_names {
        // Try to find binary in target/release or target/debug at project root
        let release_binary = project_root.join("target/release").join(&binary_name);
        let debug_binary = project_root.join("target/debug").join(&binary_name);

        let binary_path = if release_binary.exists() {
            Some(release_binary)
        } else if debug_binary.exists() {
            Some(debug_binary)
        } else {
            None
        };

        if let Some(binary_path) = binary_path {
            found_any_binary = true;

            if verbose {
                println!("  Checking binary: {}", binary_path.display());
            }

            // Check -h vs --help
            results.extend(check_help_flags(
                &binary_path,
                &binary_name,
                crate_name,
                verbose,
            ));

            // Check -V vs --version
            results.extend(check_version_flags(
                &binary_path,
                &binary_name,
                crate_name,
                verbose,
            ));
        }
    }

    if found_any_binary {
        Some(results)
    } else {
        None
    }
}

fn get_binary_names(cargo: &toml::Value) -> Vec<String> {
    let mut names = Vec::new();

    // Try [[bin]] section first
    if let Some(bins) = cargo.get("bin").and_then(|b| b.as_array()) {
        for bin in bins {
            if let Some(name) = bin.get("name").and_then(|n| n.as_str()) {
                names.push(name.to_string());
            }
        }
    }

    // If no [[bin]] sections, use package name
    if names.is_empty() {
        if let Some(name) = cargo
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        {
            names.push(name.to_string());
        }
    }

    names
}

fn check_help_flags(
    binary: &Path,
    binary_name: &str,
    crate_name: &str,
    verbose: bool,
) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label_prefix = if binary_name == crate_name {
        format!("[{}]", crate_name)
    } else {
        format!("[{}/{}]", crate_name, binary_name)
    };

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
                    format!("Help Length {}", label_prefix),
                    format!(
                        "--help ({} bytes) is longer than -h ({} bytes)",
                        long.len(),
                        short.len()
                    ),
                ));
            } else {
                results.push(CheckResult::fail(
                    format!("Help Length {}", label_prefix),
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
                    format!("AI Agent Instructions {}", label_prefix),
                    "Found AI Coding Agent section in --help",
                ));
            } else {
                results.push(CheckResult::fail(
                    format!("AI Agent Instructions {}", label_prefix),
                    "--help should include an 'AI CODING AGENT INSTRUCTIONS' section",
                ));
            }
        }
        (Err(e), _) => {
            results.push(CheckResult::fail(
                format!("Help -h {}", label_prefix),
                format!("Failed to run -h: {}", e),
            ));
        }
        (_, Err(e)) => {
            results.push(CheckResult::fail(
                format!("Help --help {}", label_prefix),
                format!("Failed to run --help: {}", e),
            ));
        }
    }

    results
}

fn check_version_flags(
    binary: &Path,
    binary_name: &str,
    crate_name: &str,
    verbose: bool,
) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label_prefix = if binary_name == crate_name {
        format!("[{}]", crate_name)
    } else {
        format!("[{}/{}]", crate_name, binary_name)
    };

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
                    format!("Version Consistency {}", label_prefix),
                    "-V and --version produce identical output",
                ));
            } else {
                results.push(CheckResult::fail(
                    format!("Version Consistency {}", label_prefix),
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
                        format!("Version Field: {} {}", field_name, label_prefix),
                        format!("Found {} in version output", field_name),
                    ));
                } else {
                    results.push(CheckResult::fail(
                        format!("Version Field: {} {}", field_name, label_prefix),
                        format!("Version output should contain {}", field_name),
                    ));
                }
            }
        }
        (Err(e), _) => {
            results.push(CheckResult::fail(
                format!("Version -V {}", label_prefix),
                format!("Failed to run -V: {}", e),
            ));
        }
        (_, Err(e)) => {
            results.push(CheckResult::fail(
                format!("Version --version {}", label_prefix),
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
    fn test_find_cargo_tomls() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let cargo_toml = temp.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"\n").unwrap();

        let found = find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("Cargo.toml"));
    }

    #[test]
    fn test_find_no_cargo_tomls() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let found = find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 0);
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

    #[test]
    fn test_get_binary_names() {
        let cargo_toml = r#"
            [package]
            name = "my-crate"
        "#;
        let cargo: toml::Value = toml::from_str(cargo_toml).unwrap();
        let names = get_binary_names(&cargo);
        assert_eq!(names, vec!["my-crate"]);
    }

    #[test]
    fn test_get_binary_names_with_bins() {
        let cargo_toml = r#"
            [package]
            name = "my-crate"

            [[bin]]
            name = "bin1"

            [[bin]]
            name = "bin2"
        "#;
        let cargo: toml::Value = toml::from_str(cargo_toml).unwrap();
        let names = get_binary_names(&cargo);
        assert_eq!(names, vec!["bin1", "bin2"]);
    }
}
