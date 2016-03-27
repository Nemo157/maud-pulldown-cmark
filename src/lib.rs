//! This library implements an adapter to allow rendering strings as markdown inside a `maud`
//! macro using `pulldown-cmark` efficiently.

#![warn(missing_docs)]
#![warn(variant_size_differences)]

#[macro_use]
extern crate html_event;
extern crate pulldown_cmark;

mod escape;
mod iter;
mod markdown;

pub use markdown::Markdown;
