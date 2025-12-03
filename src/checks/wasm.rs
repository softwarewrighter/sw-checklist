//! WASM project validation checks
//!
//! This module validates Web UI crates that use WASM. A crate is considered a Web UI
//! if it has WASM dependencies AND has web-serving indicators like:
//! - index.html file
//! - static/, public/, dist/, or assets/ directory
//! - Trunk.toml file
//!
//! WASM crates without these indicators are likely server-side WASM (sandboxes, plugins)
//! and don't need favicon/footer checks.

use super::CheckResult;
use crate::checks::tests::check_tests;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check if a crate appears to be a Web UI (not just server-side WASM)
pub fn is_web_ui_crate(crate_dir: &Path) -> bool {
    // Check for web-serving indicators
    let has_index_html = crate_dir.join("index.html").exists();
    let has_trunk_toml = crate_dir.join("Trunk.toml").exists();
    let has_static_dir = crate_dir.join("static").exists()
        || crate_dir.join("public").exists()
        || crate_dir.join("dist").exists()
        || crate_dir.join("assets").exists()
        || crate_dir.join("www").exists();

    has_index_html || has_trunk_toml || has_static_dir
}

/// Run all WASM-specific checks on a crate
pub fn check_wasm_crate(
    _project_root: &Path,
    crate_dir: &Path,
    _verbose: bool,
) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    let cargo_toml_path = crate_dir.join("Cargo.toml");
    let cargo_toml = fs::read_to_string(&cargo_toml_path)?;
    let cargo: toml::Value = toml::from_str(&cargo_toml)
        .with_context(|| format!("Failed to parse Cargo.toml at {:?}", cargo_toml_path))?;

    let crate_name = cargo
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown");

    // Check if this is actually a Web UI crate
    if !is_web_ui_crate(crate_dir) {
        // Server-side WASM - just note it and skip UI checks
        results.push(CheckResult::pass(
            format!("WASM Dependency [{}]", crate_name),
            format!(
                "{} uses WASM (server-side, no Web UI checks needed)",
                crate_name
            ),
        ));
        // Still check for tests
        results.extend(check_tests(crate_dir, crate_name, true));
        return Ok(results);
    }

    results.push(CheckResult::pass(
        format!("Web UI [{}]", crate_name),
        format!("Found Web UI crate with WASM: {}", crate_name),
    ));

    // Check for index.html
    results.extend(check_wasm_html_files(crate_dir, crate_name));

    // Check for favicon
    results.extend(check_wasm_favicon(crate_dir, crate_name));

    // Check for footer and build metadata in source code
    results.extend(check_web_ui_metadata(crate_dir, crate_name));

    // Check for tests
    results.extend(check_tests(crate_dir, crate_name, true));

    Ok(results)
}

fn check_wasm_html_files(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    // Check for index.html
    let index_html = crate_dir.join("index.html");
    if index_html.exists() {
        results.push(CheckResult::pass(
            format!("index.html {}", label),
            "Found index.html",
        ));

        // Check if it references favicon
        if let Ok(html_content) = fs::read_to_string(&index_html) {
            let html_lower = html_content.to_lowercase();
            if html_lower.contains("favicon.ico") || html_lower.contains("rel=\"icon\"") {
                results.push(CheckResult::pass(
                    format!("Favicon Reference {}", label),
                    "index.html references favicon",
                ));
            } else {
                results.push(CheckResult::fail(
                    format!("Favicon Reference {}", label),
                    "index.html should reference favicon.ico",
                ));
            }
        }
    } else {
        results.push(CheckResult::fail(
            format!("index.html {}", label),
            "WASM projects should have an index.html file",
        ));
    }

    results
}

fn check_wasm_favicon(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    let favicon = crate_dir.join("favicon.ico");
    if favicon.exists() {
        results.push(CheckResult::pass(
            format!("favicon.ico {}", label),
            "Found favicon.ico",
        ));
    } else {
        results.push(CheckResult::fail(
            format!("favicon.ico {}", label),
            "WASM projects should have a favicon.ico file",
        ));
    }

    results
}

/// Check for footer presence and build metadata across all source files.
/// This searches ALL .rs files in the crate, not just the first match,
/// because footer content might be assembled from multiple files.
fn check_web_ui_metadata(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        results.push(CheckResult::warn(
            format!("Web UI Metadata {}", label),
            "No src/ directory found",
        ));
        return results;
    }

    // Collect all .rs file contents into one searchable string
    let mut all_source = String::new();
    let mut found_footer_element = false;

    if let Ok(entries) = WalkDir::new(&src_dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rs")
                && let Ok(content) = fs::read_to_string(entry.path())
            {
                let content_lower = content.to_lowercase();
                // Check for actual footer element (not just import)
                if content_lower.contains("<footer")
                    || content_lower.contains("class=\"footer\"")
                    || (content_lower.contains("fn footer") && content_lower.contains("html!"))
                {
                    found_footer_element = true;
                }
                all_source.push_str(&content);
                all_source.push('\n');
            }
        }
    }

    // Also check index.html for footer
    let index_html = crate_dir.join("index.html");
    if let Ok(html_content) = fs::read_to_string(&index_html) {
        if html_content.to_lowercase().contains("<footer") {
            found_footer_element = true;
        }
        all_source.push_str(&html_content);
    }

    let all_lower = all_source.to_lowercase();

    // Check for footer presence
    if found_footer_element {
        results.push(CheckResult::pass(
            format!("Footer Presence {}", label),
            "Found footer element in source",
        ));
    } else if all_lower.contains("footer") {
        results.push(CheckResult::warn(
            format!("Footer Presence {}", label),
            "Found 'footer' reference but no footer element (<footer> or fn footer)",
        ));
    } else {
        results.push(CheckResult::warn(
            format!("Footer Presence {}", label),
            "No footer element found in source or HTML",
        ));
    }

    // Check for required metadata fields across all source
    // Use warnings for missing fields (content might be composed elsewhere)
    check_metadata_field(
        &mut results,
        &label,
        "Copyright",
        &all_lower,
        &["copyright"],
    );
    check_metadata_field(&mut results, &label, "License", &all_lower, &["license"]);
    check_metadata_field(
        &mut results,
        &label,
        "Repository",
        &all_lower,
        &["github.com", "gitlab.com", "repository"],
    );
    check_metadata_field(
        &mut results,
        &label,
        "Build Host",
        &all_lower,
        &["build_host", "build host"],
    );
    check_metadata_field(
        &mut results,
        &label,
        "Build Commit",
        &all_lower,
        &["build_commit", "build commit", "commit"],
    );
    check_metadata_field(
        &mut results,
        &label,
        "Build Time",
        &all_lower,
        &["build_time", "build time", "timestamp"],
    );

    results
}

/// Check for a metadata field in the combined source.
/// Uses WARN instead of FAIL because content might be composed from multiple sources.
fn check_metadata_field(
    results: &mut Vec<CheckResult>,
    label: &str,
    field_name: &str,
    content: &str,
    patterns: &[&str],
) {
    let found = patterns.iter().any(|p| content.contains(p));

    if found {
        results.push(CheckResult::pass(
            format!("{} {}", field_name, label),
            format!("Found {} reference in source", field_name),
        ));
    } else {
        results.push(CheckResult::warn(
            format!("{} {}", field_name, label),
            format!(
                "No {} reference found (check if assembled elsewhere)",
                field_name
            ),
        ));
    }
}
