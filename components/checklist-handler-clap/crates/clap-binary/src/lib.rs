//! Binary discovery and freshness checking for CLI crates

mod discover;
mod freshness;

pub use discover::{find_binary, get_binary_names};
pub use freshness::check_binary_freshness;
