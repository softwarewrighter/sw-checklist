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

    // Detect project types
    let mut has_cli = false;
    let mut has_wasm = false;
    let mut has_yew = false;

    for cargo_toml_path in &cargo_tomls {
        if let Ok(cargo_toml) = fs::read_to_string(cargo_toml_path) {
            if cargo_toml.contains("clap") {
                has_cli = true;
            }
            if cargo_toml.contains("wasm-bindgen") {
                has_wasm = true;
            }
            if cargo_toml.contains("yew") {
                has_yew = true;
            }
        }
    }

    let project_type = if has_cli && has_yew {
        "CLI + Yew"
    } else if has_cli && has_wasm {
        "CLI + WASM"
    } else if has_yew {
        "Yew (WASM)"
    } else if has_wasm {
        "WASM"
    } else if has_cli {
        "CLI"
    } else {
        "Rust Library"
    };

    println!("Project type: {}", project_type);
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

        let has_clap = cargo_toml.contains("clap");
        let is_wasm = is_wasm_crate(&cargo_toml);

        // Check crates that use clap or are WASM projects
        if has_clap {
            let crate_dir = cargo_toml_path.parent().unwrap();
            results.extend(check_rust_crate(project_root, crate_dir, verbose)?);
        } else if is_wasm {
            let crate_dir = cargo_toml_path.parent().unwrap();
            results.extend(check_wasm_crate(project_root, crate_dir, verbose)?);
        }
    }

    if results.is_empty() {
        results.push(CheckResult::pass(
            "Project Check",
            "No crates using clap or WASM found - skipping checks",
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

    // Detect if this is a WASM project
    let is_wasm = is_wasm_crate(&cargo_toml);

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

fn is_wasm_crate(cargo_toml: &str) -> bool {
    cargo_toml.contains("wasm-bindgen") || cargo_toml.contains("yew")
}

fn check_wasm_crate(
    _project_root: &Path,
    crate_dir: &Path,
    _verbose: bool,
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
        format!("WASM Dependency [{}]", crate_name),
        format!("Found WASM dependencies in {}", crate_name),
    ));

    // Check for index.html
    results.extend(check_wasm_html_files(crate_dir, crate_name));

    // Check for favicon
    results.extend(check_wasm_favicon(crate_dir, crate_name));

    // Check for footer in source code
    results.extend(check_wasm_footer_in_source(crate_dir, crate_name));

    // Check for tests
    results.extend(check_tests(crate_dir, crate_name, true));

    Ok(results)
}

fn check_wasm_html_files(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    // Check for index.html
    let index_html = crate_dir.join("index.html");
    if index_html.exists() {
        results.push(CheckResult::pass(
            format!("index.html {}", label),
            "Found index.html",
        ));

        // Check if it references favicon
        if let Ok(html_content) = fs::read_to_string(&index_html) {
            let html_lower = html_content.to_lowercase();
            if html_lower.contains("favicon.ico") || html_lower.contains("rel=\"icon\"") {
                results.push(CheckResult::pass(
                    format!("Favicon Reference {}", label),
                    "index.html references favicon",
                ));
            } else {
                results.push(CheckResult::fail(
                    format!("Favicon Reference {}", label),
                    "index.html should reference favicon.ico",
                ));
            }
        }
    } else {
        results.push(CheckResult::fail(
            format!("index.html {}", label),
            "WASM projects should have an index.html file",
        ));
    }

    results
}

fn check_wasm_favicon(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    let favicon = crate_dir.join("favicon.ico");
    if favicon.exists() {
        results.push(CheckResult::pass(
            format!("favicon.ico {}", label),
            "Found favicon.ico",
        ));
    } else {
        results.push(CheckResult::fail(
            format!("favicon.ico {}", label),
            "WASM projects should have a favicon.ico file",
        ));
    }

    results
}

fn check_wasm_footer_in_source(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    use walkdir::WalkDir;

    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    // Search for .rs files in src/
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return results;
    }

    let mut found_footer = false;
    let mut footer_content = String::new();

    // Recursively search for footer-related code
    if let Ok(entries) = WalkDir::new(&src_dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let content_lower = content.to_lowercase();
                    if content_lower.contains("footer")
                        || content_lower.contains("<footer")
                        || content_lower.contains("html! {")
                            && (content_lower.contains("copyright")
                                || content_lower.contains("license"))
                    {
                        found_footer = true;
                        footer_content = content;
                        break;
                    }
                }
            }
        }
    }

    if !found_footer {
        results.push(CheckResult::fail(
            format!("Footer Presence {}", label),
            "Could not find footer element in source code",
        ));
        return results;
    }

    results.push(CheckResult::pass(
        format!("Footer Presence {}", label),
        "Found footer in source code",
    ));

    // Check footer content
    let footer_lower = footer_content.to_lowercase();

    check_footer_field(
        &mut results,
        &label,
        "Copyright",
        &footer_lower,
        &["copyright"],
    );
    check_footer_field(
        &mut results,
        &label,
        "License Link",
        &footer_lower,
        &["license"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Repository Link",
        &footer_lower,
        &["github.com", "gitlab.com", "repository"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Build Host",
        &footer_lower,
        &["build_host", "build host", "host"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Build Commit",
        &footer_lower,
        &["build_commit", "commit", "sha"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Build Time",
        &footer_lower,
        &["build_time", "build time", "timestamp"],
    );

    results
}

fn check_footer_field(
    results: &mut Vec<CheckResult>,
    label: &str,
    field_name: &str,
    content: &str,
    patterns: &[&str],
) {
    let found = patterns.iter().any(|p| content.contains(p));

    if found {
        results.push(CheckResult::pass(
            format!("Footer {} {}", field_name, label),
            format!("Footer includes {} info", field_name),
        ));
    } else {
        results.push(CheckResult::fail(
            format!("Footer {} {}", field_name, label),
            format!("Footer should include {} info", field_name),
        ));
    }
}

fn check_tests(crate_dir: &Path, crate_name: &str, is_wasm: bool) -> Vec<CheckResult> {
    use walkdir::WalkDir;

    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    // Check for tests directory or #[cfg(test)] in source files
    let tests_dir = crate_dir.join("tests");
    let has_tests_dir = tests_dir.exists();

    // Check src/ for test modules
    let src_dir = crate_dir.join("src");
    let mut has_test_annotations = false;

    if src_dir.exists() {
        if let Ok(entries) = WalkDir::new(&src_dir)
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
        {
            for entry in entries {
                if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        if content.contains("#[test]") || content.contains("#[cfg(test)]") {
                            has_test_annotations = true;
                            break;
                        }
                    }
                }
            }
        }
    }

    if is_wasm {
        // For WASM, also check for Jest tests or curl-based tests
        let package_json = crate_dir.join("package.json");
        let has_jest = package_json.exists()
            && fs::read_to_string(&package_json)
                .map(|c| c.contains("jest"))
                .unwrap_or(false);

        if has_tests_dir || has_test_annotations || has_jest {
            results.push(CheckResult::pass(
                format!("Tests {}", label),
                "Found test files or annotations",
            ));
        } else {
            results.push(CheckResult::fail(
                format!("Tests {}", label),
                "WASM projects should have Rust tests, Jest tests, or curl-based tests",
            ));
        }
    } else {
        // For CLI projects, check for cargo tests
        if has_tests_dir || has_test_annotations {
            results.push(CheckResult::pass(
                format!("Tests {}", label),
                "Found test files or #[test] annotations",
            ));
        } else {
            results.push(CheckResult::fail(
                format!("Tests {}", label),
                "Projects should have tests directory or #[test] annotations",
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

    #[test]
    fn test_workspace_structure() {
        use tempfile::tempdir;

        // Create a workspace-like structure
        let temp = tempdir().unwrap();

        // Root workspace Cargo.toml
        let workspace_toml = temp.path().join("Cargo.toml");
        fs::write(
            &workspace_toml,
            r#"
[workspace]
members = ["crate1", "crate2"]
"#,
        )
        .unwrap();

        // Crate 1
        fs::create_dir_all(temp.path().join("crate1")).unwrap();
        let crate1_toml = temp.path().join("crate1/Cargo.toml");
        fs::write(
            &crate1_toml,
            r#"
[package]
name = "crate1"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Crate 2
        fs::create_dir_all(temp.path().join("crate2")).unwrap();
        let crate2_toml = temp.path().join("crate2/Cargo.toml");
        fs::write(
            &crate2_toml,
            r#"
[package]
name = "crate2"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Should find all 3 Cargo.toml files
        let found = find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 3);
    }

    #[test]
    fn test_nested_crates() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();

        // Root Cargo.toml
        let root_toml = temp.path().join("Cargo.toml");
        fs::write(&root_toml, "[package]\nname = \"root\"\n").unwrap();

        // Nested crate
        fs::create_dir_all(temp.path().join("nested/deep")).unwrap();
        let nested_toml = temp.path().join("nested/deep/Cargo.toml");
        fs::write(&nested_toml, "[package]\nname = \"nested\"\n").unwrap();

        // Should find both
        let found = find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 2);
    }

    #[test]
    fn test_multi_binary_crate() {
        let cargo_toml = r#"
            [package]
            name = "multi-bin"

            [[bin]]
            name = "server"
            path = "src/server.rs"

            [[bin]]
            name = "client"
            path = "src/client.rs"

            [[bin]]
            name = "admin"
            path = "src/admin.rs"
        "#;
        let cargo: toml::Value = toml::from_str(cargo_toml).unwrap();
        let names = get_binary_names(&cargo);
        assert_eq!(names, vec!["server", "client", "admin"]);
    }

    #[test]
    fn test_crate_without_clap() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let cargo_toml = temp.path().join("Cargo.toml");
        fs::write(
            &cargo_toml,
            r#"
[package]
name = "my-library"

[dependencies]
serde = "1.0"
tokio = "1.0"
"#,
        )
        .unwrap();

        let found = find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 1);

        // Should skip crates without clap/WASM and return a single pass message
        let results = run_checks(temp.path(), &found, false).unwrap();

        // Results should indicate no clap/WASM crates were found
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert!(results[0]
            .message
            .contains("No crates using clap or WASM found"));
    }

    #[test]
    fn test_empty_project() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let found = find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 0);
    }

    #[test]
    fn test_check_version_field_case_insensitive() {
        let mut results = Vec::new();
        let label = "[test]";

        // Test with "Copyright (c)" format
        check_version_field(
            &mut results,
            label,
            "Copyright",
            &"Copyright (c) 2025 Acme".to_lowercase(),
            &["copyright"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test with "Copyright:" format
        results.clear();
        check_version_field(
            &mut results,
            label,
            "Copyright",
            &"Copyright: 2025 Acme".to_lowercase(),
            &["copyright"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }

    #[test]
    fn test_check_version_field_license_variations() {
        let mut results = Vec::new();
        let label = "[test]";

        // Test "MIT License"
        check_version_field(
            &mut results,
            label,
            "License",
            &"MIT License".to_lowercase(),
            &["license", "mit", "apache", "gpl", "bsd"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test "License: MIT"
        results.clear();
        check_version_field(
            &mut results,
            label,
            "License",
            &"License: MIT".to_lowercase(),
            &["license", "mit", "apache", "gpl", "bsd"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test "Apache-2.0"
        results.clear();
        check_version_field(
            &mut results,
            label,
            "License",
            &"Apache-2.0".to_lowercase(),
            &["license", "mit", "apache", "gpl", "bsd"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test no license
        results.clear();
        check_version_field(
            &mut results,
            label,
            "License",
            &"Version 1.0.0".to_lowercase(),
            &["license", "mit", "apache", "gpl", "bsd"],
        );
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(results[0]
            .message
            .contains("does not appear to be license info"));
    }

    #[test]
    fn test_check_version_field_build_variations() {
        let mut results = Vec::new();
        let label = "[test]";

        // Test "Build Host:"
        check_version_field(
            &mut results,
            label,
            "Build Host",
            &"Build Host: x86_64-linux".to_lowercase(),
            &["build host", "build-host", "host"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test "Build-Host:"
        results.clear();
        check_version_field(
            &mut results,
            label,
            "Build Host",
            &"Build-Host: x86_64-linux".to_lowercase(),
            &["build host", "build-host", "host"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);

        // Test just "Host:"
        results.clear();
        check_version_field(
            &mut results,
            label,
            "Build Host",
            &"Host: x86_64-linux".to_lowercase(),
            &["build host", "build-host", "host"],
        );
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
    }
}
