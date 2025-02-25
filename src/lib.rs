#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
    warnings,
    deprecated_safe,
    future_incompatible,
    keyword_idents,
    let_underscore,
    nonstandard_style,
    refining_impl_trait,
    rust_2018_compatibility,
    rust_2018_idioms,
    rust_2021_compatibility,
    rust_2024_compatibility,
    unused,
    clippy::all,
    clippy::pedantic,
    clippy::style,
    clippy::perf,
    clippy::complexity,
    clippy::correctness,
    clippy::restriction,
    clippy::nursery,
    clippy::cargo
)]
#![expect(
    clippy::implicit_return,
    clippy::question_mark_used,
    clippy::else_if_without_else,
    clippy::module_name_repetitions,
    reason = "bad lint"
)]
#![expect(
    clippy::single_call_fn,
    clippy::mod_module_files,
    clippy::pub_with_shorthand,
    clippy::pattern_type_mismatch,
    reason = "style"
)]
#![expect(
    clippy::while_let_on_iterator,
    reason = "better to understand when the iterator is used after the loop breaks"
)]
#![expect(clippy::doc_include_without_cfg, reason = "see issue #13918")]
#![expect(clippy::blanket_clippy_restriction_lints, reason = "I want them all")]
#![expect(clippy::multiple_inherent_impl, reason = "useful when lots of methods")]
#![feature(coverage_attribute)]
#![feature(trivial_bounds)]

// All modules are private to prevent a breaking change after refactoring this
// crate's structure.

mod errors;
mod filter;
mod ownership;
mod parse;
pub mod prelude;
mod types;

/// A const equivalent of the [`Option::unwrap_or`] method.
const fn unwrap_or(opt: Option<bool>, default: bool) -> bool {
    match opt {
        Some(val) => val,
        None => default,
    }
}
