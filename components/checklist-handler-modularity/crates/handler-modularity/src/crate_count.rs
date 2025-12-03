//! Crate module count checking

use anyhow::Result;
use checklist_result::CheckResult;
use std::path::Path;
use walkdir::WalkDir;

/// Check crate module count
pub fn check_crate_module_count(src_dir: &Path, crate_name: &str) -> Result<Vec<CheckResult>> {
    let module_count = walk_rs_files(src_dir).count();
    let label = format!("Crate Module Count [{}]", crate_name);

    let result = if module_count > 7 {
        CheckResult::fail(
            label,
            format!("Crate {} has {} modules (max 7)", crate_name, module_count),
        )
    } else if module_count > 4 {
        CheckResult::warn(
            label,
            format!(
                "Crate {} has {} modules (warning at >4, max 7)",
                crate_name, module_count
            ),
        )
    } else {
        CheckResult::pass(label, "Crate has 4 or fewer modules")
    };
    Ok(vec![result])
}

fn walk_rs_files(dir: &Path) -> impl Iterator<Item = walkdir::DirEntry> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("rs"))
}
