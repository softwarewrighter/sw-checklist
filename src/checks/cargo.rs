//! Cargo.toml validation checks

use super::CheckResult;

/// Check that Rust edition is 2024 (required for all new projects)
pub fn check_rust_edition(cargo_toml: &str, crate_name: &str) -> CheckResult {
    // Check for edition in [package] or [workspace.package]
    let edition = extract_edition(cargo_toml);

    match edition {
        Some("2024") => CheckResult::pass(
            format!("Rust Edition [{}]", crate_name),
            "Using Rust 2024 edition",
        ),
        Some(old_edition) => CheckResult::fail(
            format!("Rust Edition [{}]", crate_name),
            format!(
                "Using Rust {} edition (must use 2024). Update edition in Cargo.toml",
                old_edition
            ),
        ),
        None => CheckResult::pass(
            format!("Rust Edition [{}]", crate_name),
            "No edition specified (inherits from workspace)",
        ),
    }
}

/// Extract edition from Cargo.toml content
fn extract_edition(cargo_toml: &str) -> Option<&str> {
    // Look for edition = "XXXX" pattern
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("edition") {
            // Extract the value between quotes
            if let Some(start) = trimmed.find('"')
                && let Some(end) = trimmed[start + 1..].find('"')
            {
                return Some(&trimmed[start + 1..start + 1 + end]);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_edition_2024_pass() {
        let toml = r#"
[package]
name = "test"
edition = "2024"
"#;
        let result = check_rust_edition(toml, "test");
        assert!(result.passed);
    }

    #[test]
    fn test_edition_2021_fail() {
        let toml = r#"
[package]
name = "test"
edition = "2021"
"#;
        let result = check_rust_edition(toml, "test");
        assert!(!result.passed);
        assert!(result.message.contains("2021"));
    }

    #[test]
    fn test_no_edition_pass() {
        let toml = r#"
[package]
name = "test"
version.workspace = true
"#;
        let result = check_rust_edition(toml, "test");
        assert!(result.passed);
    }
}
