//! Utility functions for sw-checklist

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

/// Run a command and return its stdout
pub fn run_command(binary: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new(binary)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run command: {:?} {:?}", binary, args))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Extract function name from a function definition line
pub fn extract_function_name(line: &str) -> String {
    let parts: Vec<&str> = line.split_whitespace().collect();
    for (i, &part) in parts.iter().enumerate() {
        if part == "fn" && i + 1 < parts.len() {
            let name_part = parts[i + 1];
            if let Some(open_paren) = name_part.find('(') {
                return name_part[..open_paren].to_string();
            }
            if let Some(open_angle) = name_part.find('<') {
                return name_part[..open_angle].to_string();
            }
            return name_part.to_string();
        }
    }
    "unknown".to_string()
}

/// Get the sw-install directory (typically ~/.local/softwarewrighter/bin)
pub fn get_install_dir() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|home| {
        PathBuf::from(home)
            .join(".local")
            .join("softwarewrighter")
            .join("bin")
    })
}
