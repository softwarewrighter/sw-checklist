//! Result formatting utilities

use checklist_result::{CheckResult, CheckStatus};

pub fn print_result(result: &CheckResult) {
    println!(
        "[{}] {}: {}",
        status_str(result.status),
        result.name,
        result.message
    );
}

fn status_str(status: CheckStatus) -> &'static str {
    match status {
        CheckStatus::Pass => "\x1b[32mPASS\x1b[0m",
        CheckStatus::Fail => "\x1b[31mFAIL\x1b[0m",
        CheckStatus::Warn => "\x1b[33mWARN\x1b[0m",
        CheckStatus::Info => "\x1b[36mINFO\x1b[0m",
    }
}

pub fn is_issue(status: CheckStatus) -> bool {
    matches!(status, CheckStatus::Fail | CheckStatus::Warn)
}
