//! Find Cargo.toml files in a project

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
