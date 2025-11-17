//! Clap CLI validation checks

use super::CheckResult;
use crate::checks::install::check_binary_freshness;
use crate::checks::tests::check_tests;
use crate::discovery;
use crate::utils;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Check a Rust crate that uses clap
pub fn check_rust_crate(
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

    // Detect if this is a WASM project
    let is_wasm = discovery::is_wasm_crate(&cargo_toml);

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

    // Check for tests
    results.extend(check_tests(crate_dir, crate_name, is_wasm));

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

            // Check binary freshness against installed version
            if let Ok(home) = std::env::var("HOME") {
                let installed_binary = PathBuf::from(&home)
                    .join(".local/softwarewrighter/bin")
                    .join(&binary_name);
                results.push(check_binary_freshness(
                    &binary_name,
                    &binary_path,
                    &installed_binary,
                ));
            }
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

    let short_help = utils::run_command(binary, &["-h"]);
    let long_help = utils::run_command(binary, &["--help"]);

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

    let short_version = utils::run_command(binary, &["-V"]);
    let long_version = utils::run_command(binary, &["--version"]);

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
            let version_output_lower = long.to_lowercase();

            check_version_field(
                &mut results,
                &label_prefix,
                "Copyright",
                &version_output_lower,
                &["copyright"],
            );

            check_version_field(
                &mut results,
                &label_prefix,
                "License",
                &version_output_lower,
                &["license", "mit", "apache", "gpl", "bsd"],
            );

            check_version_field(
                &mut results,
                &label_prefix,
                "Repository",
                &version_output_lower,
                &["repository", "github.com", "gitlab.com", "bitbucket.org"],
            );

            check_version_field(
                &mut results,
                &label_prefix,
                "Build Host",
                &version_output_lower,
                &["build host", "build-host", "host"],
            );

            check_version_field(
                &mut results,
                &label_prefix,
                "Build Commit",
                &version_output_lower,
                &["build commit", "build-commit", "commit", "sha", "git"],
            );

            check_version_field(
                &mut results,
                &label_prefix,
                "Build Time",
                &version_output_lower,
                &["build time", "build-time", "timestamp", "built"],
            );
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

fn check_version_field(
    results: &mut Vec<CheckResult>,
    label_prefix: &str,
    field_name: &str,
    version_output_lower: &str,
    patterns: &[&str],
) {
    let found = patterns
        .iter()
        .any(|pattern| version_output_lower.contains(pattern));

    if found {
        results.push(CheckResult::pass(
            format!("Version Field: {} {}", field_name, label_prefix),
            format!("Found {} in version output", field_name),
        ));
    } else {
        let message = if field_name == "License" {
            "There does not appear to be license info present in the -V/--version output"
                .to_string()
        } else {
            format!(
                "There does not appear to be {} info present in the -V/--version output",
                field_name
            )
        };

        results.push(CheckResult::fail(
            format!("Version Field: {} {}", field_name, label_prefix),
            message,
        ));
    }
}
