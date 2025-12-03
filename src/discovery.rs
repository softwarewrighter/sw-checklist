//! Project discovery functions

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Find all Cargo.toml files in a directory tree
pub fn find_cargo_tomls(path: &Path) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml")
        .map(|e| e.path().to_path_buf())
        .collect()
}

/// Check if a Cargo.toml indicates a WASM crate (not a workspace)
pub fn is_wasm_crate(cargo_toml: &str) -> bool {
    // Workspace Cargo.toml files may have wasm dependencies in [workspace.dependencies]
    // but they're not actual WASM crates - skip them
    if is_workspace(cargo_toml) {
        return false;
    }
    cargo_toml.contains("wasm-bindgen") || cargo_toml.contains("crate-type = [\"cdylib\"]")
}

/// Check if a Cargo.toml is a workspace (has [workspace] but no [package])
pub fn is_workspace(cargo_toml: &str) -> bool {
    cargo_toml.contains("[workspace]") && !cargo_toml.contains("[package]")
}

/// Check if a Cargo.toml has clap dependency (not in workspace.dependencies)
pub fn has_clap_dependency(cargo_toml: &str) -> bool {
    // Skip workspaces - they may declare clap in [workspace.dependencies]
    // but the actual binary crates are what we want to check
    if is_workspace(cargo_toml) {
        return false;
    }
    cargo_toml.contains("clap")
}
