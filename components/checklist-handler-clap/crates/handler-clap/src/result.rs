//! Result construction helpers

use checklist_result::CheckResult;

pub fn clap_dependency_result(crate_name: &str) -> CheckResult {
    CheckResult::pass(
        format!("Clap Dependency [{}]", crate_name),
        format!("Found clap dependency in {}", crate_name),
    )
}

pub fn no_binaries_result(crate_name: &str) -> CheckResult {
    CheckResult::fail(
        format!("Binary Check [{}]", crate_name),
        format!(
            "No built binaries for {}. Run 'cargo build --release'.",
            crate_name
        ),
    )
}
