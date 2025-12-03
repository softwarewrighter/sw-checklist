//! Version flag checking

use checklist_result::CheckResult;
use std::path::Path;

use crate::fields::check_version_fields;
use crate::util::{make_label, run_command};

/// Check -V and --version flags
pub fn check_version_flags(
    binary: &Path,
    binary_name: &str,
    crate_name: &str,
    verbose: bool,
) -> Vec<CheckResult> {
    let label = make_label(crate_name, binary_name);
    let short = run_command(binary, &["-V"]);
    let long = run_command(binary, &["--version"]);

    match (short, long) {
        (Ok(short), Ok(long)) => check_versions(&label, &short, &long, verbose),
        (Err(e), _) => vec![CheckResult::fail(
            format!("Version -V {label}"),
            format!("Failed: {e}"),
        )],
        (_, Err(e)) => vec![CheckResult::fail(
            format!("Version --version {label}"),
            format!("Failed: {e}"),
        )],
    }
}

fn check_versions(label: &str, short: &str, long: &str, verbose: bool) -> Vec<CheckResult> {
    if verbose {
        println!("  -V output: {}", short.trim());
        println!("  --version output: {}", long.trim());
    }
    let mut results = vec![check_version_consistency(label, short, long)];
    results.extend(check_version_fields(label, long));
    results
}

fn check_version_consistency(label: &str, short: &str, long: &str) -> CheckResult {
    if short == long {
        CheckResult::pass(
            format!("Version Consistency {label}"),
            "-V and --version produce identical output",
        )
    } else {
        CheckResult::fail(
            format!("Version Consistency {label}"),
            "-V and --version should produce identical output",
        )
    }
}
