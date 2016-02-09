//! This library implements an adapter to allow rendering strings as markdown inside a `maud`
//! macro using `pulldown-cmark` efficiently.

#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(trivial_casts)]
#![warn(trivial_numeric_casts)]
#![warn(unsafe_code)]
#![warn(unused_extern_crates)]
#![warn(unused_qualifications)]
#![warn(unused_results)]
#![warn(variant_size_differences)]

extern crate maud;
extern crate pulldown_cmark;

mod escape;
mod render;
mod markdown;

pub use markdown::Markdown;
