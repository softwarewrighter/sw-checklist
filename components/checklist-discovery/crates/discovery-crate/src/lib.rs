//! Crate type detection for sw-checklist
//!
//! This crate determines crate types and extracts metadata.

mod crate_type;
mod metadata;

pub use crate_type::{CrateType, detect_crate_type};
pub use metadata::extract_crate_name;
