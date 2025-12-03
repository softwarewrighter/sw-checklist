//! Configuration struct

use std::path::{Path, PathBuf};

/// Configuration for sw-checklist run
#[derive(Debug, Clone)]
pub struct Config {
    project_path: PathBuf,
    verbose: bool,
}

/// Create a new Config
pub fn new(project_path: PathBuf, verbose: bool) -> Config {
    Config {
        project_path,
        verbose,
    }
}

impl Config {
    /// Get the project root path
    pub fn project_root(&self) -> &Path {
        &self.project_path
    }

    /// Check if verbose mode is enabled
    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
