//! Main types of the API to export to external users
#![expect(clippy::pub_use, reason = "API")]

pub use crate::filter::types::Filter;
pub use crate::parse::parse_html;
pub use crate::types::html::Html;
pub use crate::types::tag::Tag;
