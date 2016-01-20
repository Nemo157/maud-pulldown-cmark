# maud-pulldown-cmark [![travis-badge][]][travis] [![release-badge][]][cargo] [![docs-badge][]][docs]

This library implements an adapter to allow rendering strings as markdown inside
a [maud][] macro using [pulldown-cmark][] efficiently.

## Example

```rust
let markdown = "
 1. A list
 2. With some
 3. Values
";

let mut buffer = String::new();

html!(buffer, {
  div {
    $(markdown::from_string(markdown))
  }
});

println!("{}", buffer);
```

```rust
let markdown = "
 1. A list
 2. With some
 3. <span>Inline html</span>
";

let events = || Parser::new(markdown).map(|ev| match ev {
  // Escape inline html
  Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
  _ => ev,
});

let mut buffer = String::new();

html!(buffer, {
  div {
    $(markdown::from_events(events))
  }
});

println!("{}", buffer);
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

[travis-badge]: https://img.shields.io/travis/Nemo157/maud-pulldown-cmark.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/maud-pulldown-cmark
[release-badge]: https://img.shields.io/github/release/Nemo157/maud-pulldown-cmark.svg?style=flat-square
[cargo]: https://crates.io/crates/maud-pulldown-cmark
[docs-badge]: https://img.shields.io/badge/API-docs-blue.svg?style=flat-square
[docs]: https://nemo157.com/maud-pulldown-cmark/
[maud]: https://github.com/lfairy/maud
[pulldown-cmark]: https://github.com/google/pulldown-cmark
