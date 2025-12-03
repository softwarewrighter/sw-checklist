//! HTML and favicon checking for Web UI crates

use checklist_result::CheckResult;
use std::fs;
use std::path::Path;

/// Check for index.html and its contents
pub fn check_html_files(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let label = format!("[{}]", crate_name);
    let index_html = crate_dir.join("index.html");

    if !index_html.exists() {
        return vec![CheckResult::fail(
            format!("index.html {}", label),
            "WASM projects should have an index.html file",
        )];
    }

    let mut results = vec![CheckResult::pass(
        format!("index.html {}", label),
        "Found index.html",
    )];
    if let Ok(html) = fs::read_to_string(&index_html) {
        results.push(check_favicon_ref(&label, &html));
    }
    results
}

/// Check for favicon.ico file
pub fn check_favicon(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let label = format!("[{}]", crate_name);
    if crate_dir.join("favicon.ico").exists() {
        vec![CheckResult::pass(
            format!("favicon.ico {}", label),
            "Found favicon.ico",
        )]
    } else {
        vec![CheckResult::fail(
            format!("favicon.ico {}", label),
            "WASM projects should have a favicon.ico file",
        )]
    }
}

fn check_favicon_ref(label: &str, html: &str) -> CheckResult {
    let lower = html.to_lowercase();
    if lower.contains("favicon.ico") || lower.contains("rel=\"icon\"") {
        CheckResult::pass(
            format!("Favicon Reference {}", label),
            "index.html references favicon",
        )
    } else {
        CheckResult::fail(
            format!("Favicon Reference {}", label),
            "index.html should reference favicon.ico",
        )
    }
}
