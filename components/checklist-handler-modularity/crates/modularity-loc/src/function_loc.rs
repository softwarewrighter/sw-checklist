//! Function LOC checking

use anyhow::Result;
use checklist_result::CheckResult;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use crate::parse::find_functions;

/// Check function LOC for all Rust files in src/
pub fn check_function_locs(src_dir: &Path, crate_name: &str) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();
    let mut any_issues = false;

    for entry in walk_rs_files(src_dir) {
        let content = fs::read_to_string(entry.path())?;
        let file_name = entry.path().file_name().unwrap().to_string_lossy();
        for (fn_name, loc) in find_functions(&content) {
            if let Some(r) = check_fn_loc(crate_name, &file_name, &fn_name, loc) {
                any_issues = true;
                results.push(r);
            }
        }
    }

    if !any_issues {
        results.push(CheckResult::pass(
            format!("Function LOC [{}]", crate_name),
            "All functions are under 25 lines",
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

fn check_fn_loc(crate_name: &str, file: &str, fn_name: &str, loc: usize) -> Option<CheckResult> {
    if loc > 50 {
        Some(CheckResult::fail(
            format!("Function LOC [{}]", crate_name),
            format!("'{}' in {} has {} lines (max 50)", fn_name, file, loc),
        ))
    } else if loc > 25 {
        Some(CheckResult::warn(
            format!("Function LOC [{}]", crate_name),
            format!("'{}' in {} has {} lines (warning >25)", fn_name, file, loc),
        ))
    } else {
        None
    }
}
