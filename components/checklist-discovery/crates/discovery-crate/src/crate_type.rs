//! Crate type detection

use discovery_cargo::{has_clap_dependency, is_wasm_crate, is_workspace};
use std::path::Path;

/// Type of crate detected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrateType {
    /// Workspace Cargo.toml (not an actual crate)
    Workspace,
    /// CLI application using clap
    Cli,
    /// WASM/Web UI crate
    Wasm,
    /// CLI + WASM combined
    CliWasm,
    /// Library crate
    Library,
}

/// Detect the type of crate from Cargo.toml content and crate directory
pub fn detect_crate_type(cargo_toml: &str, crate_dir: &Path) -> CrateType {
    if is_workspace(cargo_toml) {
        return CrateType::Workspace;
    }

    let has_clap = has_clap_dependency(cargo_toml);
    let is_binary = is_binary_crate(cargo_toml, crate_dir);
    let has_wasm = is_wasm_crate(cargo_toml);

    match (has_clap && is_binary, has_wasm) {
        (true, true) => CrateType::CliWasm,
        (true, false) => CrateType::Cli,
        (false, true) => CrateType::Wasm,
        (false, false) => CrateType::Library,
    }
}

/// Check if a crate produces a binary
fn is_binary_crate(cargo_toml: &str, crate_dir: &Path) -> bool {
    // Explicit [[bin]] section in Cargo.toml
    if cargo_toml.contains("[[bin]]") {
        return true;
    }
    // Has src/main.rs
    if crate_dir.join("src/main.rs").exists() {
        return true;
    }
    // Has src/bin/ directory with .rs files
    let bin_dir = crate_dir.join("src/bin");
    if bin_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&bin_dir) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "rs") {
                    return true;
                }
            }
        }
    }
    false
}
