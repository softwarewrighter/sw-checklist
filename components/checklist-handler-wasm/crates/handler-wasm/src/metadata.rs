//! Build metadata checking for Web UI crates

use checklist_result::CheckResult;
use wasm_html::collect_source_content;
use std::path::Path;

/// Check for footer presence and build metadata
pub fn check_web_ui_metadata(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let label = format!("[{}]", crate_name);

    if !crate_dir.join("src").exists() {
        return vec![CheckResult::warn(
            format!("Web UI Metadata {}", label),
            "No src/ directory",
        )];
    }

    let (source, found_footer) = collect_source_content(crate_dir);
    let lower = source.to_lowercase();

    let mut results = vec![check_footer(&label, found_footer, &lower)];
    results.extend(check_fields(&label, &lower));
    results
}

fn check_footer(label: &str, found: bool, lower: &str) -> CheckResult {
    if found {
        CheckResult::pass(format!("Footer Presence {}", label), "Found footer element")
    } else if lower.contains("footer") {
        CheckResult::warn(
            format!("Footer Presence {}", label),
            "Found 'footer' but no element",
        )
    } else {
        CheckResult::warn(
            format!("Footer Presence {}", label),
            "No footer element found",
        )
    }
}

fn check_fields(label: &str, content: &str) -> Vec<CheckResult> {
    vec![
        check_field(label, "Copyright", content, &["copyright"]),
        check_field(label, "License", content, &["license"]),
        check_field(
            label,
            "Repository",
            content,
            &["github.com", "gitlab.com", "repository"],
        ),
        check_field(label, "Build Host", content, &["build_host", "build host"]),
        check_field(label, "Build Commit", content, &["build_commit", "commit"]),
        check_field(label, "Build Time", content, &["build_time", "timestamp"]),
    ]
}

fn check_field(label: &str, name: &str, content: &str, patterns: &[&str]) -> CheckResult {
    if patterns.iter().any(|p| content.contains(p)) {
        CheckResult::pass(format!("{} {}", name, label), format!("Found {}", name))
    } else {
        CheckResult::warn(format!("{} {}", name, label), format!("No {} found", name))
    }
}
