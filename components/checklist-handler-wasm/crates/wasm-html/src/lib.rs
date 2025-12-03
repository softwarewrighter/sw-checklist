//! HTML/favicon checks for Web UI crates

mod html;
mod source;

pub use html::{check_favicon, check_html_files};
pub use source::collect_source_content;
