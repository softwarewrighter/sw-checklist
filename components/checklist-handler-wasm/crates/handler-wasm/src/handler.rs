//! WASM handler implementation

use anyhow::Result;
use checklist_result::CheckResult;
use discovery_crate::CrateType;
use handler_trait::{CheckContext, Handler};
use wasm_html::{check_favicon, check_html_files};

use crate::detect::is_web_ui_crate;
use crate::metadata::check_web_ui_metadata;

/// Handler for Web UI / WASM crate checks
pub struct WasmHandler;

impl Handler for WasmHandler {
    fn name(&self) -> &'static str {
        "wasm"
    }
    fn handles(&self, crate_type: CrateType) -> bool {
        crate_type == CrateType::Wasm
    }

    fn check(&self, ctx: &CheckContext) -> Result<Vec<CheckResult>> {
        if !is_web_ui_crate(ctx.crate_dir) {
            return Ok(vec![CheckResult::pass(
                format!("WASM Dependency [{}]", ctx.crate_name),
                format!("{} uses WASM (server-side)", ctx.crate_name),
            )]);
        }
        Ok(run_checks(ctx))
    }
}

fn run_checks(ctx: &CheckContext) -> Vec<CheckResult> {
    let mut r = vec![CheckResult::pass(
        format!("Web UI [{}]", ctx.crate_name),
        "Found Web UI crate",
    )];
    r.extend(check_html_files(ctx.crate_dir, ctx.crate_name));
    r.extend(check_favicon(ctx.crate_dir, ctx.crate_name));
    r.extend(check_web_ui_metadata(ctx.crate_dir, ctx.crate_name));
    r
}
