//! This library implements an adapter to allow rendering strings as markdown inside a `maud`
//! macro using `pulldown-cmark` efficiently.

#![warn(missing_docs)]
#![warn(variant_size_differences)]

extern crate maud;
extern crate pulldown_cmark;

mod escape;
mod markdown;
mod render;

pub use markdown::Markdown;
