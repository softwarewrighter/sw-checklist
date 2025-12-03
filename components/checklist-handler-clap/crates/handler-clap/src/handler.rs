//! Clap handler implementation

use anyhow::Result;
use checklist_result::CheckResult;
use discovery_crate::CrateType;
use handler_trait::{CheckContext, Handler};

use crate::check::check_crate_binaries;
use crate::result::{clap_dependency_result, no_binaries_result};

/// Handler for CLI (clap) crate checks
pub struct ClapHandler;

impl Handler for ClapHandler {
    fn name(&self) -> &'static str {
        "clap"
    }

    fn handles(&self, crate_type: CrateType) -> bool {
        crate_type == CrateType::Cli
    }

    fn check(&self, ctx: &CheckContext) -> Result<Vec<CheckResult>> {
        let mut results = vec![clap_dependency_result(ctx.crate_name)];
        match check_crate_binaries(ctx) {
            Some(r) => results.extend(r),
            None => results.push(no_binaries_result(ctx.crate_name)),
        }
        Ok(results)
    }
}
