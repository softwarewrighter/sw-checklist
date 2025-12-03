//! Version field validation

use checklist_result::CheckResult;

/// Check version output for required fields
pub fn check_version_fields(label: &str, version_output: &str) -> Vec<CheckResult> {
    let lower = version_output.to_lowercase();
    field_specs()
        .iter()
        .map(|(name, patterns)| check_field(label, name, &lower, patterns))
        .collect()
}

fn field_specs() -> Vec<(&'static str, &'static [&'static str])> {
    vec![
        ("Copyright", &["copyright"][..]),
        ("License", &["license", "mit", "apache", "gpl", "bsd"]),
        (
            "Repository",
            &["repository", "github.com", "gitlab.com", "bitbucket.org"],
        ),
        ("Build Host", &["build host", "build-host", "host"]),
        (
            "Build Commit",
            &["build commit", "build-commit", "commit", "sha", "git"],
        ),
        (
            "Build Time",
            &["build time", "build-time", "timestamp", "built"],
        ),
    ]
}

fn check_field(
    label: &str,
    field_name: &str,
    version_lower: &str,
    patterns: &[&str],
) -> CheckResult {
    if patterns.iter().any(|p| version_lower.contains(p)) {
        CheckResult::pass(
            format!("Version Field: {} {}", field_name, label),
            format!("Found {} in version output", field_name),
        )
    } else {
        CheckResult::fail(
            format!("Version Field: {} {}", field_name, label),
            format!("{} info not present in -V/--version output", field_name),
        )
    }
}
