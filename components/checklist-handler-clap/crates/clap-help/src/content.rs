//! Help content validation

use checklist_result::CheckResult;

/// Check that --help is longer than -h
pub fn check_help_length(label: &str, short: &str, long: &str) -> CheckResult {
    if long.len() > short.len() {
        CheckResult::pass(
            format!("Help Length {}", label),
            format!(
                "--help ({} bytes) is longer than -h ({} bytes)",
                long.len(),
                short.len()
            ),
        )
    } else {
        CheckResult::fail(
            format!("Help Length {}", label),
            format!(
                "--help ({} bytes) should be longer than -h ({} bytes)",
                long.len(),
                short.len()
            ),
        )
    }
}

/// Check for AI Coding Agent instructions section
pub fn check_ai_instructions(label: &str, help_output: &str) -> CheckResult {
    if help_output.contains("AI CODING AGENT") || help_output.contains("AI Coding Agent") {
        CheckResult::pass(
            format!("AI Agent Instructions {}", label),
            "Found AI Coding Agent section",
        )
    } else {
        CheckResult::fail(
            format!("AI Agent Instructions {}", label),
            "--help should include an 'AI CODING AGENT INSTRUCTIONS' section",
        )
    }
}
