//! Help flag checking

use checklist_result::CheckResult;
use std::path::Path;

use crate::content::{check_ai_instructions, check_help_length};
use crate::util::{make_label, run_command};

/// Check -h and --help flags
pub fn check_help_flags(
    binary: &Path,
    binary_name: &str,
    crate_name: &str,
    verbose: bool,
) -> Vec<CheckResult> {
    let label = make_label(crate_name, binary_name);
    let short = run_command(binary, &["-h"]);
    let long = run_command(binary, &["--help"]);

    match (short, long) {
        (Ok(short), Ok(long)) => check_help_outputs(&label, &short, &long, verbose),
        (Err(e), _) => vec![CheckResult::fail(
            format!("Help -h {label}"),
            format!("Failed: {e}"),
        )],
        (_, Err(e)) => vec![CheckResult::fail(
            format!("Help --help {label}"),
            format!("Failed: {e}"),
        )],
    }
}

fn check_help_outputs(label: &str, short: &str, long: &str, verbose: bool) -> Vec<CheckResult> {
    if verbose {
        println!("  -h output ({} bytes)", short.len());
        println!("  --help output ({} bytes)", long.len());
    }
    vec![
        check_help_length(label, short, long),
        check_ai_instructions(label, long),
    ]
}
