use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};

mod checks;
mod discovery;
mod utils;

use checks::CheckResult;

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
    - AI Coding Agent instructions in help\n\n\
    For all Rust projects, it validates modularity:\n\
    - Function LOC: warns >25 lines, fails >50 lines\n\
    - Module function count: warns >4 functions, fails >7 functions\n\
    - Crate module count: warns >4 modules, fails >7 modules\n\
    - Project crate count: warns >4 crates, fails >7 crates"
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

  Rust projects with clap:
  - Help and version output validation
  - AI Coding Agent instructions in --help
  - Version metadata (copyright, license, repository, build info)

  All Rust projects (modularity checks):
  - Functions: warns if >25 LOC, fails if >50 LOC
  - Modules: warns if >4 functions, fails if >7 functions
  - Crates: warns if >4 modules, fails if >7 modules
  - Projects: warns if >4 crates, fails if >7 crates

  WASM projects:
  - Frontend validation checks (index.html, favicon, footer)

  All projects:
  - sw-install presence check (warning if not installed)

MODULARITY PHILOSOPHY:

Small, focused units are easier to understand, test, and maintain.
The 7±2 rule (Miller's Law) guides these limits:
- Functions should do one thing (< 25 LOC ideal)
- Modules should have a clear purpose (≤ 4 functions ideal)
- Crates should be cohesive (≤ 4 modules ideal)
- Projects should be well-scoped (≤ 4 crates ideal)

For more information, see the repository:
https://github.com/softwarewrighter/sw-checklist
"#;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let project_path = fs::canonicalize(&cli.project_path)
        .with_context(|| format!("Failed to access project path: {:?}", cli.project_path))?;

    println!("Checking project: {}", project_path.display());
    println!();

    // Find all Cargo.toml files in the project
    let cargo_tomls = discovery::find_cargo_tomls(&project_path);

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

    let mut results = run_checks(&project_path, &cargo_tomls, cli.verbose)?;

    // Add sw-install presence check
    results.push(checks::install::check_sw_install_presence());

    // Add project crate count check (exclude workspace Cargo.toml files)
    let crate_count = cargo_tomls
        .iter()
        .filter(|path| {
            if let Ok(content) = fs::read_to_string(path) {
                !discovery::is_workspace(&content)
            } else {
                true // count if we can't read it
            }
        })
        .count();
    if crate_count > 7 {
        results.push(CheckResult::fail(
            "Project Crate Count",
            format!("Project has {} crates (max 7)", crate_count),
        ));
    } else if crate_count > 4 {
        results.push(CheckResult::warn(
            "Project Crate Count",
            format!("Project has {} crates (warning at >4, max 7)", crate_count),
        ));
    } else {
        results.push(CheckResult::pass(
            "Project Crate Count",
            format!("Project has {} crates (4 or fewer)", crate_count),
        ));
    }

    print_results(&results);

    let passed = results.iter().filter(|r| r.passed && !r.is_warning).count();
    let failed = results.iter().filter(|r| !r.passed).count();
    let warnings = results.iter().filter(|r| r.is_warning).count();

    println!();
    if warnings > 0 {
        println!(
            "Summary: {} passed, {} failed, {} warnings",
            passed, failed, warnings
        );
    } else {
        println!("Summary: {} passed, {} failed", passed, failed);
    }

    if failed > 0 {
        std::process::exit(1);
    }

    Ok(())
}

fn run_checks(
    project_root: &Path,
    cargo_tomls: &[PathBuf],
    verbose: bool,
) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    for cargo_toml_path in cargo_tomls {
        let cargo_toml = fs::read_to_string(cargo_toml_path)
            .with_context(|| format!("Failed to read Cargo.toml at {:?}", cargo_toml_path))?;

        let cargo: toml::Value = toml::from_str(&cargo_toml)
            .with_context(|| format!("Failed to parse Cargo.toml at {:?}", cargo_toml_path))?;

        let crate_name = cargo
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown");

        let is_workspace = discovery::is_workspace(&cargo_toml);
        let has_clap = discovery::has_clap_dependency(&cargo_toml);
        let is_wasm = discovery::is_wasm_crate(&cargo_toml);

        // Determine crate type for verbose output
        let crate_type = if is_workspace {
            "workspace"
        } else if has_clap && is_wasm {
            "CLI + WASM"
        } else if has_clap {
            "CLI (clap)"
        } else if is_wasm {
            "WASM"
        } else {
            "library"
        };

        if verbose {
            println!(
                "Checking: {} [{}] ({})",
                cargo_toml_path.display(),
                crate_name,
                crate_type
            );
        }

        // Skip workspace Cargo.toml for clap/wasm checks (they're just containers)
        if is_workspace {
            if verbose {
                println!("  Skipping clap/wasm checks for workspace");
            }
            continue;
        }

        // Check crates that use clap or are WASM projects
        if has_clap {
            let crate_dir = cargo_toml_path.parent().unwrap();
            if verbose {
                println!("  Running CLI (clap) checks for {}", crate_name);
            }
            results.extend(checks::clap::check_rust_crate(
                project_root,
                crate_dir,
                verbose,
            )?);
        } else if is_wasm {
            let crate_dir = cargo_toml_path.parent().unwrap();
            if verbose {
                println!("  Running WASM checks for {}", crate_name);
            }
            results.extend(checks::wasm::check_wasm_crate(
                project_root,
                crate_dir,
                verbose,
            )?);
        }

        // Run modularity checks on all crates (not workspaces)
        let crate_dir = cargo_toml_path.parent().unwrap();
        if verbose {
            println!("  Running modularity checks for {}", crate_name);
        }
        results.extend(checks::modularity::check_modularity(crate_dir, crate_name)?);
    }

    if results.is_empty() {
        results.push(CheckResult::pass(
            "Project Check",
            "No crates found - skipping checks",
        ));
    }

    Ok(results)
}

fn print_results(results: &[CheckResult]) {
    println!("Check Results:");
    println!("{}", "=".repeat(80));

    for result in results {
        let status = if result.is_warning {
            "⚠ WARN"
        } else if result.passed {
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

        let found = discovery::find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 1);
        assert!(found[0].ends_with("Cargo.toml"));
    }

    #[test]
    fn test_find_no_cargo_tomls() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let found = discovery::find_cargo_tomls(temp.path());
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
        let found = discovery::find_cargo_tomls(temp.path());
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
        let found = discovery::find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 2);
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

        let found = discovery::find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 1);

        // Should skip clap/WASM checks but still run modularity checks
        let results = run_checks(temp.path(), &found, false).unwrap();

        // Should have modularity check results (at least one)
        assert!(!results.is_empty());
        // All modularity checks should pass for an empty library
        assert!(results.iter().all(|r| r.passed || r.is_warning));
    }

    #[test]
    fn test_empty_project() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let found = discovery::find_cargo_tomls(temp.path());
        assert_eq!(found.len(), 0);
    }

    #[test]
    fn test_check_result_warn() {
        let warn = CheckResult::warn("Test Warning", "This is a warning");
        assert!(warn.is_warning);
        assert_eq!(warn.name, "Test Warning");
        assert_eq!(warn.message, "This is a warning");
    }

    #[test]
    fn test_check_sw_install_present() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();

        // Test when sw-install does not exist
        let result = checks::install::check_sw_install_presence_impl(Some(temp.path()));
        assert!(result.is_warning);
        assert!(result.message.contains("sw-install"));
        assert!(result.message.contains("not installed"));

        // Test when sw-install exists
        let sw_install_path = temp.path().join("sw-install");
        fs::write(&sw_install_path, "fake sw-install").unwrap();

        let result = checks::install::check_sw_install_presence_impl(Some(temp.path()));
        assert!(result.passed);
        assert!(!result.is_warning);
        assert_eq!(result.message, "sw-install is installed");
    }

    #[test]
    fn test_check_binary_freshness_no_installed_binary() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();

        // Create a fake binary in target/release
        let target_dir = temp.path().join("target/release");
        fs::create_dir_all(&target_dir).unwrap();
        let local_binary = target_dir.join("test-binary");
        fs::write(&local_binary, "test").unwrap();

        // Non-existent installed binary
        let installed_binary = PathBuf::from("/nonexistent/path/test-binary");

        let result = checks::install::check_binary_freshness(
            "test-binary",
            &local_binary,
            &installed_binary,
        );

        // Should not warn if installed binary doesn't exist
        assert!(!result.is_warning);
        assert!(result.passed);
    }

    #[test]
    fn test_check_binary_freshness_local_newer() {
        use std::thread;
        use std::time::Duration;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();

        // Create installed binary first
        let installed_dir = temp.path().join("installed");
        fs::create_dir_all(&installed_dir).unwrap();
        let installed_binary = installed_dir.join("test-binary");
        fs::write(&installed_binary, "old").unwrap();

        // Wait a bit to ensure different timestamps
        thread::sleep(Duration::from_millis(10));

        // Create newer local binary
        let local_dir = temp.path().join("target/release");
        fs::create_dir_all(&local_dir).unwrap();
        let local_binary = local_dir.join("test-binary");
        fs::write(&local_binary, "new").unwrap();

        let result = checks::install::check_binary_freshness(
            "test-binary",
            &local_binary,
            &installed_binary,
        );

        // Should warn because local is newer
        assert!(result.is_warning);
        assert!(result.message.contains("newer"));
        assert!(result.message.contains("acceptance test"));
        assert!(result.message.contains("sw-install"));
    }

    #[test]
    fn test_check_binary_freshness_installed_newer() {
        use std::thread;
        use std::time::Duration;
        use tempfile::tempdir;

        let temp = tempdir().unwrap();

        // Create local binary first
        let local_dir = temp.path().join("target/release");
        fs::create_dir_all(&local_dir).unwrap();
        let local_binary = local_dir.join("test-binary");
        fs::write(&local_binary, "old").unwrap();

        // Wait a bit to ensure different timestamps
        thread::sleep(Duration::from_millis(10));

        // Create newer installed binary
        let installed_dir = temp.path().join("installed");
        fs::create_dir_all(&installed_dir).unwrap();
        let installed_binary = installed_dir.join("test-binary");
        fs::write(&installed_binary, "new").unwrap();

        let result = checks::install::check_binary_freshness(
            "test-binary",
            &local_binary,
            &installed_binary,
        );

        // Should not warn because installed is newer or equal
        assert!(!result.is_warning);
        assert!(result.passed);
    }

    // Modularity tests

    #[test]
    fn test_function_loc_under_25_pass() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        // Create Cargo.toml
        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create lib.rs with small function (10 lines)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub fn small_function() {
    let x = 1;
    let y = 2;
    let z = x + y;
    println!("{}", z);
}
"#,
        )
        .unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        // Should pass with no warnings
        let function_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Function LOC"))
            .collect();

        assert!(!function_results.is_empty());
        assert!(function_results.iter().all(|r| r.passed));
        assert!(function_results.iter().all(|r| !r.is_warning));
    }

    #[test]
    fn test_function_loc_26_to_50_warn() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create function with 30 lines (warning)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::from("pub fn medium_function() {\n");
        for i in 0..28 {
            content.push_str(&format!("    let x{} = {};\n", i, i));
        }
        content.push_str("}\n");

        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let function_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Function LOC"))
            .collect();

        assert!(!function_results.is_empty());
        // Should have a warning for the large function
        assert!(function_results.iter().any(|r| r.is_warning));
    }

    #[test]
    fn test_function_loc_over_50_fail() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create function with 60 lines (fail)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::from("pub fn huge_function() {\n");
        for i in 0..58 {
            content.push_str(&format!("    let x{} = {};\n", i, i));
        }
        content.push_str("}\n");

        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let function_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Function LOC"))
            .collect();

        assert!(!function_results.is_empty());
        // Should fail for the huge function
        assert!(function_results.iter().any(|r| !r.passed));
    }

    #[test]
    fn test_module_function_count_under_4_pass() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create module with 3 functions
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            src_dir.join("lib.rs"),
            r#"
pub fn func1() {}
pub fn func2() {}
pub fn func3() {}
"#,
        )
        .unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let module_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Module Function Count"))
            .collect();

        assert!(!module_results.is_empty());
        assert!(module_results.iter().all(|r| r.passed));
        assert!(module_results.iter().all(|r| !r.is_warning));
    }

    #[test]
    fn test_module_function_count_5_to_7_warn() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create module with 5 functions (warning)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::new();
        for i in 1..=5 {
            content.push_str(&format!("pub fn func{}() {{}}\n", i));
        }
        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let module_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Module Function Count"))
            .collect();

        assert!(!module_results.is_empty());
        assert!(module_results.iter().any(|r| r.is_warning));
    }

    #[test]
    fn test_module_function_count_over_7_fail() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create module with 8 functions (fail)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::new();
        for i in 1..=8 {
            content.push_str(&format!("pub fn func{}() {{}}\n", i));
        }
        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let module_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Module Function Count"))
            .collect();

        assert!(!module_results.is_empty());
        assert!(module_results.iter().any(|r| !r.passed));
    }

    #[test]
    fn test_crate_module_count_under_4_pass() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create 3 module files
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "pub mod module1;\npub mod module2;").unwrap();
        fs::write(src_dir.join("module1.rs"), "pub fn func() {}").unwrap();
        fs::write(src_dir.join("module2.rs"), "pub fn func() {}").unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let crate_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Crate Module Count"))
            .collect();

        assert!(!crate_results.is_empty());
        assert!(crate_results.iter().all(|r| r.passed));
        assert!(crate_results.iter().all(|r| !r.is_warning));
    }

    #[test]
    fn test_crate_module_count_over_7_fail() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create 8 module files (fail)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut lib_content = String::new();
        for i in 1..=8 {
            lib_content.push_str(&format!("pub mod module{};\n", i));
            fs::write(src_dir.join(format!("module{}.rs", i)), "pub fn func() {}").unwrap();
        }
        fs::write(src_dir.join("lib.rs"), lib_content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let crate_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("Crate Module Count"))
            .collect();

        assert!(!crate_results.is_empty());
        assert!(crate_results.iter().any(|r| !r.passed));
    }

    // File LOC tests

    #[test]
    fn test_file_loc_under_350_pass() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create file with 100 lines (pass)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::new();
        for i in 0..100 {
            content.push_str(&format!("// Line {}\n", i));
        }
        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let file_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("File LOC"))
            .collect();

        assert!(!file_results.is_empty());
        assert!(file_results.iter().all(|r| r.passed));
        assert!(file_results.iter().all(|r| !r.is_warning));
    }

    #[test]
    fn test_file_loc_351_to_500_warn() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create file with 400 lines (warning)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::new();
        for i in 0..400 {
            content.push_str(&format!("// Line {}\n", i));
        }
        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let file_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("File LOC"))
            .collect();

        assert!(!file_results.is_empty());
        assert!(file_results.iter().any(|r| r.is_warning));
        assert!(file_results[0].message.contains("400 lines"));
    }

    #[test]
    fn test_file_loc_over_500_fail() {
        use tempfile::tempdir;

        let temp = tempdir().unwrap();
        let crate_dir = temp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();

        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"
[package]
name = "test-crate"
version = "0.1.0"
"#,
        )
        .unwrap();

        // Create file with 600 lines (fail)
        let src_dir = crate_dir.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let mut content = String::new();
        for i in 0..600 {
            content.push_str(&format!("// Line {}\n", i));
        }
        fs::write(src_dir.join("lib.rs"), content).unwrap();

        let results = checks::modularity::check_modularity(&crate_dir, "test-crate").unwrap();

        let file_results: Vec<_> = results
            .iter()
            .filter(|r| r.name.contains("File LOC"))
            .collect();

        assert!(!file_results.is_empty());
        assert!(file_results.iter().any(|r| !r.passed));
        assert!(file_results[0].message.contains("600 lines"));
        assert!(file_results[0].message.contains("Extract modules"));
    }
}
