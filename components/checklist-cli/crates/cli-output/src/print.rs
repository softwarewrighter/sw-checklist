//! Print functions for check results

use checklist_config::Config;
use checklist_result::{CheckResult, CheckStatus};

use crate::format::{is_issue, print_result};

const MAX_ISSUES_TO_SHOW: usize = 5;

/// Print per-check results (all in verbose mode, issues only otherwise)
pub fn print_results(results: &[CheckResult], config: &Config) {
    if config.verbose() {
        results.iter().for_each(print_result);
    } else {
        print_issues_summary(results);
    }
}

fn print_issues_summary(results: &[CheckResult]) {
    let issues: Vec<_> = results.iter().filter(|r| is_issue(r.status)).collect();
    match issues.len() {
        0 => {}
        1..=MAX_ISSUES_TO_SHOW => issues.iter().for_each(|r| print_result(r)),
        _ => println!("Run with -v/--verbose for details"),
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
