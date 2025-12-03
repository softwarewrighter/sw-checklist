//! Cargo.toml discovery for sw-checklist
//!
//! This crate finds and classifies Cargo.toml files in a project.

mod classify;
mod find;

pub use classify::{has_clap_dependency, is_wasm_crate, is_workspace};
pub use find::find_cargo_tomls;
