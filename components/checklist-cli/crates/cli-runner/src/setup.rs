//! Runner setup utilities

use handler_trait::Handler;
use std::path::Path;

/// Create all check handlers
pub fn create_handlers() -> Vec<Box<dyn Handler>> {
    vec![
        Box::new(handler_cargo::CargoHandler),
        Box::new(handler_modularity::ModularityHandler),
        Box::new(handler_clap::ClapHandler),
        Box::new(handler_wasm::WasmHandler),
    ]
}

/// Extract crate name from Cargo.toml content
pub fn extract_crate_name(cargo_toml: &str, crate_dir: &Path) -> String {
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("name")
            && trimmed.contains('=')
            && let Some(start) = trimmed.find('"')
            && let Some(end) = trimmed[start + 1..].find('"')
        {
            return trimmed[start + 1..start + 1 + end].to_string();
        }
    }
    crate_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string()
}
