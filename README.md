# maud-pulldown-cmark [![downloads-badge][] ![release-badge][]][crate] [![license-badge][]](#license)

This library implements an adapter to allow rendering strings as markdown inside
a [maud][] macro using [pulldown-cmark][] efficiently.

[downloads-badge]: https://img.shields.io/crates/d/maud-pulldown-cmark.svg?style=flat-square
[release-badge]: https://img.shields.io/crates/v/maud-pulldown-cmark.svg?style=flat-square
[license-badge]: https://img.shields.io/crates/l/maud-pulldown-cmark.svg?style=flat-square
[crate]: https://crates.io/crates/maud-pulldown-cmark

[maud]: https://github.com/lfairy/maud
[pulldown-cmark]: https://github.com/google/pulldown-cmark

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
    $(Markdown::from_string(markdown))
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
