//! Edition extraction from Cargo.toml

/// Extract edition from Cargo.toml content
pub fn extract_edition(cargo_toml: &str) -> Option<&str> {
    for line in cargo_toml.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("edition")
            && let Some(start) = trimmed.find('"')
            && let Some(end) = trimmed[start + 1..].find('"')
        {
            return Some(&trimmed[start + 1..start + 1 + end]);
        }
    }
    None
}
