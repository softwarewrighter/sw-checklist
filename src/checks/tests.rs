//! Test validation checks

use super::CheckResult;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check if a crate has tests
pub fn check_tests(crate_dir: &Path, crate_name: &str, is_wasm: bool) -> Vec<CheckResult> {
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
