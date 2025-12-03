//! Crate metadata extraction

/// Extract crate name from parsed Cargo.toml
pub fn extract_crate_name(cargo: &toml::Value) -> &str {
    cargo
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("unknown")
}
