//! Modularity check handler for sw-checklist
//!
//! Checks function LOC, file LOC, module counts, etc.

mod crate_count;
mod handler;
mod module_count;

pub use handler::ModularityHandler;
