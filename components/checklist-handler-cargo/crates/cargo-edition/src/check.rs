//! Rust edition check

use checklist_result::CheckResult;

use crate::extract::extract_edition;

/// Check that Rust edition is 2024 (required for all new projects)
pub fn check_rust_edition(cargo_toml: &str, crate_name: &str) -> CheckResult {
    let label = format!("Rust Edition [{}]", crate_name);
    match extract_edition(cargo_toml) {
        Some("2024") => CheckResult::pass(label, "Using Rust 2024 edition"),
        Some(old) => CheckResult::fail(label, format!("Using Rust {} edition (must use 2024)", old)),
        None => CheckResult::pass(label, "No edition specified (inherits from workspace)"),
    }
}
