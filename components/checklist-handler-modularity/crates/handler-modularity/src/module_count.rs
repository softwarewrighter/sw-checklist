//! Module function count checking

use anyhow::Result;
use checklist_result::CheckResult;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check module function counts
pub fn check_module_function_counts(src_dir: &Path, crate_name: &str) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();
    let mut any_issues = false;

    for entry in walk_rs_files(src_dir) {
        let path = entry.path();
        let content = fs::read_to_string(path)?;
        let file_name = path.file_name().unwrap().to_string_lossy();
        if let Some(r) = check_module_fn_count(crate_name, &file_name, count_functions(&content)) {
            any_issues = true;
            results.push(r);
        }
    }

    if !any_issues {
        results.push(CheckResult::pass(
            format!("Module Function Count [{}]", crate_name),
            "All modules have 4 or fewer functions",
        ));
    }
    Ok(results)
}

fn check_module_fn_count(
    crate_name: &str,
    file_name: &str,
    fn_count: usize,
) -> Option<CheckResult> {
    let label = format!("Module Function Count [{}]", crate_name);
    if fn_count > 7 {
        Some(CheckResult::fail(
            label,
            format!("Module {} has {} functions (max 7)", file_name, fn_count),
        ))
    } else if fn_count > 4 {
        Some(CheckResult::warn(
            label,
            format!(
                "Module {} has {} functions (warning at >4, max 7)",
                file_name, fn_count
            ),
        ))
    } else {
        None
    }
}

fn walk_rs_files(dir: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
}

fn count_functions(content: &str) -> usize {
    content
        .lines()
        .filter(|line| {
            let t = line.trim();
            t.starts_with("fn ")
                || t.starts_with("pub fn ")
                || t.starts_with("async fn ")
                || t.starts_with("pub async fn ")
        })
        .count()
}
