//! Classify Cargo.toml files

/// Check if a Cargo.toml is a workspace (has [workspace] but no [package])
pub fn is_workspace(cargo_toml: &str) -> bool {
    cargo_toml.contains("[workspace]") && !cargo_toml.contains("[package]")
}

/// Check if a Cargo.toml has clap dependency (not in workspace.dependencies)
pub fn has_clap_dependency(cargo_toml: &str) -> bool {
    if is_workspace(cargo_toml) {
        return false;
    }
    cargo_toml.contains("clap")
}

/// Check if a Cargo.toml indicates a WASM crate (not a workspace)
pub fn is_wasm_crate(cargo_toml: &str) -> bool {
    if is_workspace(cargo_toml) {
        return false;
    }
    cargo_toml.contains("wasm-bindgen") || cargo_toml.contains("crate-type = [\"cdylib\"]")
}
