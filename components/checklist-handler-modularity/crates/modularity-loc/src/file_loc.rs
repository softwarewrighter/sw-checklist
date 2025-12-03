//! File LOC checking

use anyhow::Result;
use checklist_result::CheckResult;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check file LOC for all Rust files in src/
pub fn check_file_locs(src_dir: &Path, crate_name: &str) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();
    let mut any_issues = false;

    for entry in walk_rs_files(src_dir) {
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        if let Some(result) = check_file(path, &content, crate_name) {
            any_issues = true;
            results.push(result);
        }
    }

    if !any_issues {
        results.push(CheckResult::pass(
            format!("File LOC [{}]", crate_name),
            "All files are 350 or fewer lines",
        ));
    }
    Ok(results)
}

fn walk_rs_files(dir: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
}

fn check_file(path: &Path, content: &str, crate_name: &str) -> Option<CheckResult> {
    let file_name = path.file_name().unwrap().to_string_lossy();
    let loc = content.lines().count();

    if loc > 500 {
        Some(CheckResult::fail(
            format!("File LOC [{}]", crate_name),
            format!("{} has {} lines (max 500)", file_name, loc),
        ))
    } else if loc > 350 {
        Some(CheckResult::warn(
            format!("File LOC [{}]", crate_name),
            format!("{} has {} lines (warning >350)", file_name, loc),
        ))
    } else {
        None
    }
}
