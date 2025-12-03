//! Configuration builder

use crate::config::Config;
use std::path::PathBuf;

/// Builder for Config
#[derive(Debug, Default)]
pub struct ConfigBuilder {
    project_path: Option<PathBuf>,
    verbose: bool,
}

impl ConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the project path
    pub fn project_path(mut self, path: PathBuf) -> Self {
        self.project_path = Some(path);
        self
    }

    /// Set verbose mode
    pub fn verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    /// Build the Config
    pub fn build(self) -> Config {
        let path = self.project_path.unwrap_or_else(|| PathBuf::from("."));
        crate::config::new(path, self.verbose)
    }
}
