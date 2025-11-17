//! WASM project validation checks

use super::CheckResult;
use crate::checks::tests::check_tests;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

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

    results.push(CheckResult::pass(
        format!("WASM Dependency [{}]", crate_name),
        format!("Found WASM dependencies in {}", crate_name),
    ));

    // Check for index.html
    results.extend(check_wasm_html_files(crate_dir, crate_name));

    // Check for favicon
    results.extend(check_wasm_favicon(crate_dir, crate_name));

    // Check for footer in source code
    results.extend(check_wasm_footer_in_source(crate_dir, crate_name));

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

fn check_wasm_footer_in_source(crate_dir: &Path, crate_name: &str) -> Vec<CheckResult> {
    let mut results = Vec::new();
    let label = format!("[{}]", crate_name);

    // Search for .rs files in src/
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return results;
    }

    let mut found_footer = false;
    let mut footer_content = String::new();

    // Recursively search for footer-related code
    if let Ok(entries) = WalkDir::new(&src_dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("rs") {
                if let Ok(content) = fs::read_to_string(entry.path()) {
                    let content_lower = content.to_lowercase();
                    if content_lower.contains("footer")
                        || content_lower.contains("<footer")
                        || content_lower.contains("html! {")
                            && (content_lower.contains("copyright")
                                || content_lower.contains("license"))
                    {
                        found_footer = true;
                        footer_content = content;
                        break;
                    }
                }
            }
        }
    }

    if !found_footer {
        results.push(CheckResult::fail(
            format!("Footer Presence {}", label),
            "Could not find footer element in source code",
        ));
        return results;
    }

    results.push(CheckResult::pass(
        format!("Footer Presence {}", label),
        "Found footer in source code",
    ));

    // Check footer content
    let footer_lower = footer_content.to_lowercase();

    check_footer_field(
        &mut results,
        &label,
        "Copyright",
        &footer_lower,
        &["copyright"],
    );
    check_footer_field(
        &mut results,
        &label,
        "License Link",
        &footer_lower,
        &["license"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Repository Link",
        &footer_lower,
        &["github.com", "gitlab.com", "repository"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Build Host",
        &footer_lower,
        &["build_host", "build host", "host"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Build Commit",
        &footer_lower,
        &["build_commit", "commit", "sha"],
    );
    check_footer_field(
        &mut results,
        &label,
        "Build Time",
        &footer_lower,
        &["build_time", "build time", "timestamp"],
    );

    results
}

fn check_footer_field(
    results: &mut Vec<CheckResult>,
    label: &str,
    field_name: &str,
    content: &str,
    patterns: &[&str],
) {
    let found = patterns.iter().any(|p| content.contains(p));

    if found {
        results.push(CheckResult::pass(
            format!("Footer {} {}", field_name, label),
            format!("Footer includes {} info", field_name),
        ));
    } else {
        results.push(CheckResult::fail(
            format!("Footer {} {}", field_name, label),
            format!("Footer should include {} info", field_name),
        ));
    }
}
