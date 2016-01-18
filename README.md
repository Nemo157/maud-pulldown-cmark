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
    $(Markdown::FromString(markdown))
  }
});

printf!("{}", buffer);
```

[travis-badge]: https://img.shields.io/travis/Nemo157/maud-pulldown-cmark.svg?style=flat-square
[travis]: https://travis-ci.org/Nemo157/maud-pulldown-cmark
[release-badge]: https://img.shields.io/github/release/Nemo157/maud-pulldown-cmark.svg?style=flat-square
[cargo]: https://crates.io/crates/maud-pulldown-cmark
[docs-badge]: https://img.shields.io/badge/API-docs-blue.svg?style=flat-square
[docs]: https://nemo157.com/maud-pulldown-cmark/
[maud]: https://github.com/lfairy/maud
[pulldown-cmark]: https://github.com/google/pulldown-cmark
