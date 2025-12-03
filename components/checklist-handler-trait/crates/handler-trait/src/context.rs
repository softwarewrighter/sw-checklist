//! Check context passed to handlers

use checklist_config::Config;
use discovery_crate::CrateType;
use std::path::Path;

/// Context for a check operation
pub struct CheckContext<'a> {
    /// Global configuration
    pub config: &'a Config,
    /// Path to the crate directory
    pub crate_dir: &'a Path,
    /// Name of the crate
    pub crate_name: &'a str,
    /// Type of crate
    pub crate_type: CrateType,
    /// Raw Cargo.toml content
    pub cargo_toml: &'a str,
}
