use std::fmt;
use std::marker::PhantomData;

use maud::RenderOnce;
use pulldown_cmark::{ Parser, Event };

use render;

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
/// # use maud_pulldown_cmark::Markdown;
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
///     $(Markdown::from_string(markdown))
///   }
/// }).unwrap();
///
/// println!("{}", buffer);
/// # }
/// ```
///
#[cfg_attr(feature = "nightly", doc = " ```")]
#[cfg_attr(not(feature = "nightly"), doc = " ```ignore")]
/// # #![feature(plugin)]
/// # #![plugin(maud_macros)]
/// # extern crate maud;
/// # extern crate pulldown_cmark;
/// # extern crate maud_pulldown_cmark;
/// # use pulldown_cmark::{ Parser, Event };
/// # use maud_pulldown_cmark::Markdown;
/// # fn main() {
/// let markdown = "
///  1. A list
///  2. With some
///  3. <span>Inline html</span>
/// ";
///
/// let events = Parser::new(markdown).map(|ev| match ev {
///   // Escape inline html
///   Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
///   _ => ev,
/// });
///
/// let mut buffer = String::new();
///
/// html!(buffer, {
///   div {
///     $(Markdown::from_events(events))
///   }
/// }).unwrap();
///
/// println!("{}", buffer);
/// # }
/// ```
pub struct Markdown<'a, I: 'a + Iterator<Item=Event<'a>>> {
  events: I,
  phantom: PhantomData<&'a I>,
}

impl<'a> Markdown<'a, Parser<'a>> {
  /// To allow rendering from a string.
  pub fn from_string(s: &'a str) -> Markdown<'a, Parser<'a>> {
    Markdown {
      events: Parser::new(s),
      phantom: PhantomData,
    }
  }
}

impl<'a, I: 'a + Iterator<Item=Event<'a>>> Markdown<'a, I> {
  /// To allow rendering from a stream of events (useful for modifying the output of the general parser).
  pub fn from_events(events: I) -> Markdown<'a, I> {
    Markdown {
      events: events,
      phantom: PhantomData,
    }
  }
}

impl<'a, I: 'a + Iterator<Item=Event<'a>>> RenderOnce for Markdown<'a, I> {
  fn render(self, w: &mut fmt::Write) -> fmt::Result {
    render::render_events(self.events, w)
  }
}

#[cfg(test)]
mod tests {
  #[test]
  pub fn test_from_string() {
    use super::Markdown;
    use maud::RenderOnce;

    let markdown = "
 1. A list
 2. With some
 3. <span>Inline html</span>
    ";

    let mut buffer = String::new();
    Markdown::from_string(markdown).render(&mut buffer).unwrap();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li><span>Inline html</span></li>\n</ol>\n");
  }

  #[test]
  pub fn test_from_events() {
    use super::Markdown;
    use maud::RenderOnce;
    use pulldown_cmark::{ Parser, Event };

    let markdown = "
 1. A list
 2. With some
 3. <span>Inline html</span>
    ";

    let events = Parser::new(markdown).map(|ev| match ev {
      // Escape inline html
      Event::Html(html) | Event::InlineHtml(html) => Event::Text(html),
      _ => ev,
    });

    let mut buffer = String::new();
    Markdown::from_events(events).render(&mut buffer).unwrap();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li>&lt;span&gt;Inline html&lt;/span&gt;</li>\n</ol>\n");
  }
}
