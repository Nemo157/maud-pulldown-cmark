# maud-pulldown-cmark [![downloads-badge][] ![release-badge][]][crate] [![license-badge][]](#license)

This library implements an adapter to allow rendering markdown strings inside a
[maud][] macro using [pulldown-cmark][] efficiently.

[downloads-badge]: https://img.shields.io/crates/d/maud-pulldown-cmark.svg?style=flat-square
[release-badge]: https://img.shields.io/crates/v/maud-pulldown-cmark.svg?style=flat-square
[license-badge]: https://img.shields.io/crates/l/maud-pulldown-cmark.svg?style=flat-square
[crate]: https://crates.io/crates/maud-pulldown-cmark

[maud]: https://github.com/lfairy/maud
[pulldown-cmark]: https://github.com/google/pulldown-cmark

## Example

```rust
#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;
extern crate maud_pulldown_cmark;

use maud_pulldown_cmark::Markdown;

fn main() {
    let markdown = "
1. A list
2. With some
3. Values";

    let mut buffer = String::new();

    html!(buffer, {
        div {
            ^(Markdown::from_string(markdown))
        }
    }).unwrap();

    println!("{}", buffer);
}
```

```rust
#![feature(plugin)]
#![plugin(maud_macros)]

extern crate maud;
extern crate maud_pulldown_cmark;
extern crate pulldown_cmark;

use maud_pulldown_cmark::Markdown;
use pulldown_cmark::{Parser, Event};

fn main() {
    let markdown = "
1. A list
2. With some
3. <span>Inline html</span>";

    let events = Parser::new(markdown).map(|ev| match ev {
        // Escape inline html
        Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
        _ => ev,
    });

    let mut buffer = String::new();

    html!(buffer, {
        div {
            ^(Markdown::from_events(events))
        }
    }).unwrap();

    println!("{}", buffer);
}
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
