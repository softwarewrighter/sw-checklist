//! Installation and binary freshness checks

use super::CheckResult;
use crate::utils;
use std::fs;
use std::path::Path;

/// Check if sw-install is present in the standard location
pub fn check_sw_install_presence() -> CheckResult {
    check_sw_install_presence_impl(None)
}

/// Check if sw-install is present (testable version with custom directory)
pub fn check_sw_install_presence_impl(install_dir: Option<&Path>) -> CheckResult {
    let install_dir = match install_dir {
        Some(dir) => dir.to_path_buf(),
        None => match utils::get_install_dir() {
            Some(dir) => dir,
            None => {
                return CheckResult::warn("sw-install Check", "Could not determine HOME directory");
            }
        },
    };

    let sw_install_path = install_dir.join("sw-install");

    if sw_install_path.exists() {
        CheckResult::pass("sw-install Check", "sw-install is installed")
    } else {
        CheckResult::warn(
            "sw-install Check",
            "sw-install is not installed. Install from: https://github.com/softwarewrighter/sw-install",
        )
    }
}

/// Check if a local binary is newer than its installed version
pub fn check_binary_freshness(
    binary_name: &str,
    local_binary: &Path,
    installed_binary: &Path,
) -> CheckResult {
    // Only warn if the installed binary exists
    if !installed_binary.exists() {
        return CheckResult::pass(
            format!("Binary Freshness [{}]", binary_name),
            format!("{} not yet installed", binary_name),
        );
    }

    // Compare modification times
    let local_metadata = match fs::metadata(local_binary) {
        Ok(m) => m,
        Err(_) => {
            return CheckResult::pass(
                format!("Binary Freshness [{}]", binary_name),
                "Could not check local binary metadata",
            );
        }
    };

    let installed_metadata = match fs::metadata(installed_binary) {
        Ok(m) => m,
        Err(_) => {
            return CheckResult::pass(
                format!("Binary Freshness [{}]", binary_name),
                "Could not check installed binary metadata",
            );
        }
    };

    let local_modified = match local_metadata.modified() {
        Ok(t) => t,
        Err(_) => {
            return CheckResult::pass(
                format!("Binary Freshness [{}]", binary_name),
                "Could not determine local binary modification time",
            );
        }
    };

    let installed_modified = match installed_metadata.modified() {
        Ok(t) => t,
        Err(_) => {
            return CheckResult::pass(
                format!("Binary Freshness [{}]", binary_name),
                "Could not determine installed binary modification time",
            );
        }
    };

    if local_modified > installed_modified {
        CheckResult::warn(
            format!("Binary Freshness [{}]", binary_name),
            "Local build is newer than installed version. \
                Manually acceptance test the updated project and reinstall via sw-install"
                .to_string(),
        )
    } else {
        CheckResult::pass(
            format!("Binary Freshness [{}]", binary_name),
            format!("{} is up to date", binary_name),
        )
    }
}
