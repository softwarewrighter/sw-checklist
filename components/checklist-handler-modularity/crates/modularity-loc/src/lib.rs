//! LOC checking for modularity handler

mod file_loc;
mod function_loc;
mod parse;

pub use file_loc::check_file_locs;
pub use function_loc::check_function_locs;
