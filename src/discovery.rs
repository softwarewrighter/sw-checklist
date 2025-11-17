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

/// Check if a Cargo.toml indicates a WASM crate
pub fn is_wasm_crate(cargo_toml: &str) -> bool {
    cargo_toml.contains("wasm-bindgen") || cargo_toml.contains("crate-type = [\"cdylib\"]")
}
