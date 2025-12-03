//! Binary checking orchestration

use checklist_result::CheckResult;
use clap_binary::{check_binary_freshness, find_binary, get_binary_names};
use clap_help::check_help_flags;
use clap_version::check_version_flags;
use handler_trait::CheckContext;
use std::path::Path;

/// Check binaries for a crate
pub fn check_crate_binaries(ctx: &CheckContext) -> Option<Vec<CheckResult>> {
    let mut results = Vec::new();
    let mut found_any = false;

    for binary_name in get_binary_names(ctx.cargo_toml, ctx.crate_name) {
        if let Some(path) = find_binary(ctx.config.project_root(), &binary_name) {
            found_any = true;
            results.extend(check_binary(ctx, &path, &binary_name));
        }
    }
    found_any.then_some(results)
}

fn check_binary(ctx: &CheckContext, path: &Path, binary_name: &str) -> Vec<CheckResult> {
    if ctx.config.verbose() {
        println!("  Checking binary: {}", path.display());
    }
    let mut results = check_help_flags(path, binary_name, ctx.crate_name, ctx.config.verbose());
    results.extend(check_version_flags(
        path,
        binary_name,
        ctx.crate_name,
        ctx.config.verbose(),
    ));
    results.push(check_binary_freshness(binary_name, path));
    results
}
