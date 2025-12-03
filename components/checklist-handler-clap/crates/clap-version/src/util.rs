//! Utility functions for version checking

use std::path::Path;
use std::process::Command;

pub fn run_command(binary: &Path, args: &[&str]) -> Result<String, String> {
    Command::new(binary)
        .args(args)
        .output()
        .map_err(|e| e.to_string())
        .and_then(|output| String::from_utf8(output.stdout).map_err(|e| e.to_string()))
}

pub fn make_label(crate_name: &str, binary_name: &str) -> String {
    if binary_name == crate_name {
        format!("[{}]", crate_name)
    } else {
        format!("[{}/{}]", crate_name, binary_name)
    }
}
