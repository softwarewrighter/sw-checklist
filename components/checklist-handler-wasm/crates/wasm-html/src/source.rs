//! Source content collection for Web UI crates

use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Collect all source content and check for footer patterns
pub fn collect_source_content(crate_dir: &Path) -> (String, bool) {
    let (rs_content, found_rs) = collect_rs_files(crate_dir);
    let (html_content, found_html) = collect_index_html(crate_dir);

    let mut all = rs_content;
    all.push_str(&html_content);
    (all, found_rs || found_html)
}

fn collect_rs_files(crate_dir: &Path) -> (String, bool) {
    let mut content = String::new();
    let mut found = false;

    for entry in WalkDir::new(crate_dir.join("src"))
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        if let Ok(file) = fs::read_to_string(entry.path()) {
            if has_footer(&file) {
                found = true;
            }
            content.push_str(&file);
            content.push('\n');
        }
    }
    (content, found)
}

fn collect_index_html(crate_dir: &Path) -> (String, bool) {
    if let Ok(html) = fs::read_to_string(crate_dir.join("index.html")) {
        let found = html.to_lowercase().contains("<footer");
        (html, found)
    } else {
        (String::new(), false)
    }
}

fn has_footer(content: &str) -> bool {
    let lower = content.to_lowercase();
    lower.contains("<footer")
        || lower.contains("class=\"footer\"")
        || (lower.contains("fn footer") && lower.contains("html!"))
}
