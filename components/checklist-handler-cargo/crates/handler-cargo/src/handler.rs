//! Cargo handler implementation

use anyhow::Result;
use cargo_edition::check_rust_edition;
use checklist_result::CheckResult;
use discovery_crate::CrateType;
use handler_trait::{CheckContext, Handler};

/// Handler for Cargo.toml checks
pub struct CargoHandler;

impl Handler for CargoHandler {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn handles(&self, _crate_type: CrateType) -> bool {
        true
    }

    fn check(&self, ctx: &CheckContext) -> Result<Vec<CheckResult>> {
        Ok(vec![check_rust_edition(ctx.cargo_toml, ctx.crate_name)])
    }
}
