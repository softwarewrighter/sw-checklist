//! Modularity handler implementation

use anyhow::Result;
use checklist_result::CheckResult;
use discovery_crate::CrateType;
use modularity_loc::{check_file_locs, check_function_locs};
use handler_trait::{CheckContext, Handler};

use crate::crate_count::check_crate_module_count;
use crate::module_count::check_module_function_counts;

/// Handler for modularity checks
pub struct ModularityHandler;

impl Handler for ModularityHandler {
    fn name(&self) -> &'static str {
        "modularity"
    }

    fn handles(&self, crate_type: CrateType) -> bool {
        // Run on all crates except workspaces
        crate_type != CrateType::Workspace
    }

    fn check(&self, ctx: &CheckContext) -> Result<Vec<CheckResult>> {
        let mut results = Vec::new();
        let src_dir = ctx.crate_dir.join("src");

        if !src_dir.exists() {
            return Ok(vec![CheckResult::pass(
                format!("Modularity [{}]", ctx.crate_name),
                "No src/ directory found",
            )]);
        }

        // Check function LOC
        results.extend(check_function_locs(&src_dir, ctx.crate_name)?);

        // Check file LOC
        results.extend(check_file_locs(&src_dir, ctx.crate_name)?);

        // Check module function counts
        results.extend(check_module_function_counts(&src_dir, ctx.crate_name)?);

        // Check crate module count
        results.extend(check_crate_module_count(&src_dir, ctx.crate_name)?);

        Ok(results)
    }
}
