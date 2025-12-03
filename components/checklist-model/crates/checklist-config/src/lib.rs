//! Configuration types for sw-checklist
//!
//! This crate provides configuration structures built from CLI arguments.

mod builder;
mod config;

pub use builder::ConfigBuilder;
pub use config::Config;
