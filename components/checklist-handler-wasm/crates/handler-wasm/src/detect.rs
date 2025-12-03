//! Web UI detection

use std::path::Path;

/// Check if a crate appears to be a Web UI (not just server-side WASM)
pub fn is_web_ui_crate(crate_dir: &Path) -> bool {
    let has_index = crate_dir.join("index.html").exists();
    let has_trunk = crate_dir.join("Trunk.toml").exists();
    let has_static = ["static", "public", "dist", "assets", "www"]
        .iter()
        .any(|d| crate_dir.join(d).exists());
    has_index || has_trunk || has_static
}
