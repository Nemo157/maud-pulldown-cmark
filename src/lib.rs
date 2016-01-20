//! This library implements an adapter to allow rendering strings as markdown inside a `maud`
//! macro using `pulldown-cmark` efficiently.

#![warn(missing_docs)]
#![warn(variant_size_differences)]

extern crate maud;
extern crate pulldown_cmark;

/// The public constructors for the adapters
pub mod markdown;

mod escape;
mod render;
