//! Binary freshness checking

use checklist_result::CheckResult;
use std::path::{Path, PathBuf};

/// Check if built binary is fresher than installed version
pub fn check_binary_freshness(binary_name: &str, built_binary: &Path) -> CheckResult {
    let label = format!("Binary Freshness [{}]", binary_name);
    let Some(installed) = get_installed_path(binary_name) else {
        return CheckResult::warn(label, "Could not determine HOME directory");
    };

    if !installed.exists() {
        return CheckResult::warn(
            label,
            format!("{} is not installed (run sw-install)", binary_name),
        );
    }

    compare_timestamps(&label, built_binary, &installed)
}

fn get_installed_path(binary_name: &str) -> Option<PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(
        PathBuf::from(home)
            .join(".local/softwarewrighter/bin")
            .join(binary_name),
    )
}

fn compare_timestamps(label: &str, built: &Path, installed: &Path) -> CheckResult {
    let built_time = built.metadata().and_then(|m| m.modified()).ok();
    let installed_time = installed.metadata().and_then(|m| m.modified()).ok();
    match (built_time, installed_time) {
        (Some(b), Some(i)) if b > i => {
            CheckResult::warn(label, "Built binary is newer (run sw-install to update)")
        }
        (Some(_), Some(_)) => CheckResult::pass(label, "Installed binary is up to date"),
        _ => CheckResult::warn(label, "Could not compare binary timestamps"),
    }
}
