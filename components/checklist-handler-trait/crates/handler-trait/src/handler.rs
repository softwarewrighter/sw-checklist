//! Handler trait definition

use crate::context::CheckContext;
use anyhow::Result;
use checklist_result::CheckResult;
use discovery_crate::CrateType;

/// Trait for check handlers
pub trait Handler {
    /// Name of the handler
    fn name(&self) -> &'static str;

    /// Check if this handler should run for the given crate type
    fn handles(&self, crate_type: CrateType) -> bool;

    /// Run the checks and return results
    fn check(&self, ctx: &CheckContext) -> Result<Vec<CheckResult>>;
}
