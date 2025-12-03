//! Output formatting for check results

use checklist_result::{CheckResult, CheckStatus};

/// Print all check results
pub fn print_results(results: &[CheckResult]) {
    for result in results {
        let status_str = match result.status {
            CheckStatus::Pass => "\x1b[32mPASS\x1b[0m",
            CheckStatus::Fail => "\x1b[31mFAIL\x1b[0m",
            CheckStatus::Warn => "\x1b[33mWARN\x1b[0m",
            CheckStatus::Info => "\x1b[36mINFO\x1b[0m",
        };
        println!("[{}] {}: {}", status_str, result.name, result.message);
    }
}

/// Print summary of results
pub fn print_summary(results: &[CheckResult]) {
    let passed = results
        .iter()
        .filter(|r| r.status == CheckStatus::Pass)
        .count();
    let failed = results
        .iter()
        .filter(|r| r.status == CheckStatus::Fail)
        .count();
    let warnings = results
        .iter()
        .filter(|r| r.status == CheckStatus::Warn)
        .count();
    let info = results
        .iter()
        .filter(|r| r.status == CheckStatus::Info)
        .count();

    println!();
    println!(
        "Summary: {} passed, {} failed, {} warnings, {} info",
        passed, failed, warnings, info
    );
}
