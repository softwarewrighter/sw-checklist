//! Modularity validation checks

use super::CheckResult;
use crate::utils;
use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Check modularity constraints for a crate
pub fn check_modularity(crate_dir: &Path, crate_name: &str) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();

    // Find all .rs files in src/
    let src_dir = crate_dir.join("src");
    if !src_dir.exists() {
        return Ok(vec![CheckResult::pass(
            format!("Modularity [{}]", crate_name),
            "No src/ directory found, skipping modularity checks",
        )]);
    }

    let mut module_function_counts: HashMap<String, usize> = HashMap::new();
    let mut module_count = 0;

    // Check each Rust file
    for entry in WalkDir::new(&src_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
    {
        let file_path = entry.path();
        let content = fs::read_to_string(file_path)?;

        // Simple line-based function detection
        let lines: Vec<&str> = content.lines().collect();
        let file_name = file_path.file_name().unwrap().to_string_lossy().to_string();
        let file_loc = lines.len();

        // Check file LOC
        if file_loc > 500 {
            results.push(CheckResult::fail(
                format!("File LOC [{}]", crate_name),
                format!(
                    "File {} has {} lines (max 500). Consider: Extract modules, separate structs/impls, or use traits",
                    file_name, file_loc
                ),
            ));
        } else if file_loc > 350 {
            results.push(CheckResult::warn(
                format!("File LOC [{}]", crate_name),
                format!(
                    "File {} has {} lines (warning at >350, max 500). Consider: Extract related functionality to separate files",
                    file_name, file_loc
                ),
            ));
        }

        module_count += 1;
        let mut function_count = 0;

        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();

            // Look for function definitions
            if line.starts_with("fn ")
                || line.starts_with("pub fn ")
                || line.starts_with("async fn ")
                || line.starts_with("pub async fn ")
            {
                function_count += 1;

                // Extract function name
                let fn_name = utils::extract_function_name(line);

                // Find the opening brace
                let mut brace_line = i;
                let mut found_open_brace = false;
                while brace_line < lines.len() && brace_line < i + 10 {
                    if lines[brace_line].contains('{') {
                        found_open_brace = true;
                        break;
                    }
                    brace_line += 1;
                }

                if !found_open_brace {
                    i += 1;
                    continue;
                }

                // Count braces to find the end
                let mut brace_count = 0;
                let mut end_line = brace_line;
                for (idx, line) in lines.iter().enumerate().skip(brace_line) {
                    for ch in line.chars() {
                        if ch == '{' {
                            brace_count += 1;
                        } else if ch == '}' {
                            brace_count -= 1;
                            if brace_count == 0 {
                                end_line = idx;
                                break;
                            }
                        }
                    }
                    if brace_count == 0 {
                        break;
                    }
                }

                let loc = end_line - i + 1;

                if loc > 50 {
                    results.push(CheckResult::fail(
                        format!("Function LOC [{}]", crate_name),
                        format!(
                            "Function '{}' in {} has {} lines (max 50)",
                            fn_name, file_name, loc
                        ),
                    ));
                } else if loc > 25 {
                    results.push(CheckResult::warn(
                        format!("Function LOC [{}]", crate_name),
                        format!(
                            "Function '{}' in {} has {} lines (warning at >25, max 50)",
                            fn_name, file_name, loc
                        ),
                    ));
                }

                i = end_line + 1;
            } else {
                i += 1;
            }
        }

        module_function_counts.insert(file_name.clone(), function_count);

        // Check module function count
        if function_count > 7 {
            results.push(CheckResult::fail(
                format!("Module Function Count [{}]", crate_name),
                format!(
                    "Module {} has {} functions (max 7)",
                    file_name, function_count
                ),
            ));
        } else if function_count > 4 {
            results.push(CheckResult::warn(
                format!("Module Function Count [{}]", crate_name),
                format!(
                    "Module {} has {} functions (warning at >4, max 7)",
                    file_name, function_count
                ),
            ));
        }
    }

    // Check crate module count
    if module_count > 7 {
        results.push(CheckResult::fail(
            format!("Crate Module Count [{}]", crate_name),
            format!("Crate {} has {} modules (max 7)", crate_name, module_count),
        ));
    } else if module_count > 4 {
        results.push(CheckResult::warn(
            format!("Crate Module Count [{}]", crate_name),
            format!(
                "Crate {} has {} modules (warning at >4, max 7)",
                crate_name, module_count
            ),
        ));
    }

    // If no issues found, add pass results
    if !results.iter().any(|r| r.name.contains("Function LOC")) {
        results.push(CheckResult::pass(
            format!("Function LOC [{}]", crate_name),
            "All functions are under 25 lines",
        ));
    }

    if !results
        .iter()
        .any(|r| r.name.contains("Module Function Count"))
    {
        results.push(CheckResult::pass(
            format!("Module Function Count [{}]", crate_name),
            "All modules have 4 or fewer functions",
        ));
    }

    if !results
        .iter()
        .any(|r| r.name.contains("Crate Module Count"))
    {
        results.push(CheckResult::pass(
            format!("Crate Module Count [{}]", crate_name),
            "Crate has 4 or fewer modules",
        ));
    }

    if !results.iter().any(|r| r.name.contains("File LOC")) {
        results.push(CheckResult::pass(
            format!("File LOC [{}]", crate_name),
            "All files are 350 or fewer lines",
        ));
    }

    Ok(results)
}
