//! Output formatting for check results

use checklist_config::Config;
use checklist_result::{CheckResult, CheckStatus};

/// Print per-check results (only in verbose mode)
pub fn print_results(results: &[CheckResult], config: &Config) {
    if !config.verbose() {
        return;
    }
    for result in results {
        println!(
            "[{}] {}: {}",
            status_str(result.status),
            result.name,
            result.message
        );
    }
}

fn status_str(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "\x1b[32mPASS\x1b[0m",
        CheckStatus::Fail => "\x1b[31mFAIL\x1b[0m",
        CheckStatus::Warn => "\x1b[33mWARN\x1b[0m",
        CheckStatus::Info => "\x1b[36mINFO\x1b[0m",
    }
}

/// Print summary of results
pub fn print_summary(results: &[CheckResult]) {
    let (passed, failed, warnings, info) = count_results(results);
    println!("Summary: {passed} passed, {failed} failed, {warnings} warnings, {info} info");
}

fn count_results(results: &[CheckResult]) -> (usize, usize, usize, usize) {
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
    (passed, failed, warnings, info)
}
