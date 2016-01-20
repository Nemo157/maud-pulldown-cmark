use std::marker::PhantomData;
use pulldown_cmark::{ Parser, Event };

/// The adapter that allows rendering markdown inside a `maud` macro.
///
/// # Examples
///
#[cfg_attr(feature = "nightly", doc = " ```")]
#[cfg_attr(not(feature = "nightly"), doc = " ```ignore")]
/// # #![feature(plugin)]
/// # #![plugin(maud_macros)]
/// # extern crate maud;
/// # extern crate maud_pulldown_cmark;
/// # use maud_pulldown_cmark::markdown;
/// # fn main() {
/// let markdown = "
///  1. A list
///  2. With some
///  3. <span>Inline html</span>
/// ";
///
/// let mut buffer = String::new();
///
/// html!(buffer, {
///   div {
///     $(markdown::from_string(markdown))
///   }
/// }).unwrap();
///
/// println!("{}", buffer);
/// # }
/// ```
pub struct MarkdownString<'a>(&'a str);

/// The adapter that allows rendering an iterator of markdown events inside a `maud` macro.
///
/// # Examples
///
#[cfg_attr(feature = "nightly", doc = " ```")]
#[cfg_attr(not(feature = "nightly"), doc = " ```ignore")]
/// # #![feature(plugin)]
/// # #![plugin(maud_macros)]
/// # extern crate maud;
/// # extern crate pulldown_cmark;
/// # extern crate maud_pulldown_cmark;
/// # use pulldown_cmark::{ Parser, Event };
/// # use maud_pulldown_cmark::markdown;
/// # fn main() {
/// let markdown = "
///  1. A list
///  2. With some
///  3. <span>Inline html</span>
/// ";
///
/// let events = || Parser::new(markdown).map(|ev| match ev {
///   // Escape inline html
///   Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
///   _ => ev,
/// });
///
/// let mut buffer = String::new();
///
/// html!(buffer, {
///   div {
///     $(markdown::from_events(&events))
///   }
/// }).unwrap();
///
/// println!("{}", buffer);
/// # }
/// ```
pub struct MarkdownEvents<'a, I: 'a + Iterator<Item=Event<'a>>, F: Fn() -> I>(F, PhantomData<&'a I>);

/// To allow rendering from a string.
pub fn from_string<'a>(s: &'a str) -> MarkdownString<'a> {
  MarkdownString(s)
}

/// To allow rendering from a stream of events (useful for modifying the output of the general parser).
pub fn from_events<'a, I: 'a + Iterator<Item=Event<'a>>, F: Fn() -> I>(events: F) -> MarkdownEvents<'a, I, F> {
  MarkdownEvents(events, PhantomData)
}

impl<'a> MarkdownString<'a> {
  /// To get a parser wrapping the provided string.
  pub fn events(&self) -> Parser<'a> {
    Parser::new(self.0)
  }
}

impl<'a, I: 'a + Iterator<Item=Event<'a>>, F: Fn() -> I> MarkdownEvents<'a, I, F> {
  /// To get a copy of the provided events.
  pub fn events(&self) -> I {
    let f = &self.0;
    f()
  }
}

#[cfg(test)]
mod tests {
  #[test]
  pub fn test_from_string() {
    use maud::Render;

    let markdown = "
 1. A list
 2. With some
 3. <span>Inline html</span>
    ";

    let mut buffer = String::new();
    super::from_string(markdown).render(&mut buffer).unwrap();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li><span>Inline html</span></li>\n</ol>\n");
  }

  #[test]
  pub fn test_from_events() {
    use maud::Render;
    use pulldown_cmark::{ Parser, Event };

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
    super::from_events(events).render(&mut buffer).unwrap();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li>&lt;span&gt;Inline html&lt;/span&gt;</li>\n</ol>\n");
  }
}
