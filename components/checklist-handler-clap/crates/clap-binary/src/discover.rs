//! Binary discovery utilities

use std::path::{Path, PathBuf};

/// Get binary names from Cargo.toml
pub fn get_binary_names(cargo_toml: &str, crate_name: &str) -> Vec<String> {
    let mut names = Vec::new();
    if let Ok(cargo) = cargo_toml.parse::<toml::Value>()
        && let Some(bins) = cargo.get("bin").and_then(|b| b.as_array())
    {
        for bin in bins {
            if let Some(name) = bin.get("name").and_then(|n| n.as_str()) {
                names.push(name.to_string());
            }
        }
    }
    if names.is_empty() {
        names.push(crate_name.to_string());
    }
    names
}

/// Find binary in target directories (root or component)
pub fn find_binary(project_root: &Path, binary_name: &str) -> Option<PathBuf> {
    // Check root target directory
    if let Some(path) = find_in_target(project_root, binary_name) {
        return Some(path);
    }
    // Check component target directories
    find_in_components(project_root, binary_name)
}

fn find_in_target(dir: &Path, binary_name: &str) -> Option<PathBuf> {
    let release = dir.join("target/release").join(binary_name);
    if release.exists() {
        return Some(release);
    }
    let debug = dir.join("target/debug").join(binary_name);
    if debug.exists() {
        return Some(debug);
    }
    None
}

fn find_in_components(project_root: &Path, binary_name: &str) -> Option<PathBuf> {
    let components_dir = project_root.join("components");
    if !components_dir.is_dir() {
        return None;
    }
    let Ok(entries) = std::fs::read_dir(&components_dir) else {
        return None;
    };
    for entry in entries.flatten() {
        if entry.path().is_dir()
            && let Some(path) = find_in_target(&entry.path(), binary_name)
        {
            return Some(path);
        }
    }
    None
}
