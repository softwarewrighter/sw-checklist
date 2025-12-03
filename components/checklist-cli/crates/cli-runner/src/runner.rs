//! Main runner logic

use anyhow::Result;
use checklist_config::Config;
use checklist_result::CheckResult;
use discovery_cargo::find_cargo_tomls;
use discovery_crate::detect_crate_type;
use handler_trait::{CheckContext, Handler};
use std::fs;
use std::path::Path;

use crate::setup::{create_handlers, extract_crate_name};
use cli_output::{print_results, print_summary};

/// Run all checks and return exit code
pub fn run(config: &Config) -> Result<i32> {
    let cargo_tomls = find_cargo_tomls(config.project_root());

    if cargo_tomls.is_empty() {
        println!("No Cargo.toml files found in {:?}", config.project_root());
        return Ok(1);
    }

    let results = check_all_crates(config, &cargo_tomls)?;
    print_results(&results, config);
    if config.verbose() {
        println!();
    }
    print_summary(&results);

    let failed = results.iter().filter(|r| !r.status.passed()).count();
    Ok(if failed > 0 { 1 } else { 0 })
}

fn check_all_crates(
    config: &Config,
    cargo_tomls: &[std::path::PathBuf],
) -> Result<Vec<CheckResult>> {
    let handlers = create_handlers();
    let mut results = Vec::new();
    for cargo_path in cargo_tomls {
        results.extend(check_crate(config, cargo_path, &handlers)?);
    }
    Ok(results)
}

fn check_crate(
    config: &Config,
    cargo_path: &Path,
    handlers: &[Box<dyn Handler>],
) -> Result<Vec<CheckResult>> {
    let cargo_toml = fs::read_to_string(cargo_path)?;
    let crate_dir = cargo_path.parent().unwrap();
    let crate_type = detect_crate_type(&cargo_toml, crate_dir);
    let crate_name = extract_crate_name(&cargo_toml, crate_dir);

    if config.verbose() {
        println!("Checking {} ({:?})", crate_name, crate_type);
    }

    let ctx = CheckContext {
        config,
        crate_dir,
        crate_name: &crate_name,
        crate_type,
        cargo_toml: &cargo_toml,
    };
    run_handlers(&ctx, handlers)
}

fn run_handlers(ctx: &CheckContext, handlers: &[Box<dyn Handler>]) -> Result<Vec<CheckResult>> {
    let mut results = Vec::new();
    for handler in handlers {
        if handler.handles(ctx.crate_type) {
            results.extend(handler.check(ctx)?);
        }
    }
    Ok(results)
}
