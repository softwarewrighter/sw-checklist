//! Handler trait for sw-checklist checks
//!
//! This crate defines the Handler trait used by all check handlers.

mod context;
mod handler;

pub use context::CheckContext;
pub use handler::Handler;
