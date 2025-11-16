use chrono::Utc;
use std::process::Command;

fn main() {
    // Get git commit SHA
    let commit_sha = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .ok()
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .unwrap_or_else(|| "unknown".to_string())
        .trim()
        .to_string();

    // Get build timestamp
    let build_timestamp = Utc::now().to_rfc3339();

    // Get build host
    let build_host = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "unknown".to_string());

    // Pass to compiler
    println!("cargo:rustc-env=BUILD_COMMIT_SHA={}", commit_sha);
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
    println!("cargo:rustc-env=BUILD_HOST={}", build_host);

    // Re-run if git HEAD changes
    println!("cargo:rerun-if-changed=.git/HEAD");
}
