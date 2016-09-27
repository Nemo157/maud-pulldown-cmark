use std::fmt;
use std::marker::PhantomData;

use maud::RenderOnce;
use pulldown_cmark::{ Parser, Event };

use render;

/// The adapter that allows rendering markdown inside a `maud` macro.
///
/// # Examples
///
/// ```
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
/// let buffer = html! {
///   div {
///     (Markdown::from_string(markdown))
///   }
/// };
///
/// println!("{}", buffer.into_string());
/// # }
/// ```
///
/// ```
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
/// let buffer = html! {
///   div {
///     (Markdown::from_events(events))
///   }
/// };
///
/// println!("{}", buffer.into_string());
/// # }
/// ```
pub struct Markdown<'a, I: 'a + Iterator<Item=Event<'a>>> {
  events: I,
  config: render::Config,
  phantom: PhantomData<&'a I>,
}

impl<'a> Markdown<'a, Parser<'a>> {
  /// To allow rendering from a string.
  pub fn from_string(s: &'a str) -> Markdown<'a, Parser<'a>> {
    Markdown {
      events: Parser::new(s),
      config: render::Config {
        header_ids: false,
      },
      phantom: PhantomData,
    }
  }
}

impl<'a, I: 'a + Iterator<Item=Event<'a>>> Markdown<'a, I> {
  /// To allow rendering from a stream of events (useful for modifying the output of the general parser).
  pub fn from_events(events: I) -> Markdown<'a, I> {
    Markdown {
      events: events,
      config: render::Config {
        header_ids: false,
      },
      phantom: PhantomData,
    }
  }

  /// Generate ids for all headers, lowercases the text in the header and replaces spaces with -.
  ///
  /// # Examples
  ///
  /// ```
  /// # #![feature(plugin)]
  /// # #![plugin(maud_macros)]
  /// # extern crate maud;
  /// # extern crate maud_pulldown_cmark;
  /// # use maud_pulldown_cmark::Markdown;
  /// # fn main() {
  // TODO: Work out how to escape the # so this can go on the next line
  /// let markdown = "# Header
  /// ## A Sub Header
  /// ";
  ///
  /// let buffer = html!(
  ///   (Markdown::from_string(markdown).with_header_ids())
  /// );
  ///
  /// assert_eq!(buffer.into_string(), "<h1 id=\"header\">Header</h1>\n<h2 id=\"a-sub-header\">A Sub Header</h2>\n");
  /// # }
  /// ```
  pub fn with_header_ids(self) -> Markdown<'a, I> {
    Markdown {
      config: render::Config {
        header_ids: true,
        ..self.config
      },
      ..self
    }
  }
}

impl<'a, I: 'a + Iterator<Item=Event<'a>>> RenderOnce for Markdown<'a, I> {
  fn render_once(self, w: &mut fmt::Write) -> fmt::Result {
    render::render_events(self.config, self.events, w)
  }
}

#[cfg(test)]
mod tests {
  use super::Markdown;
  use maud::RenderOnce;
  use pulldown_cmark::{ Parser, Event };

  impl<'a, I: 'a + Iterator<Item=Event<'a>>> Markdown<'a, I> {
    pub fn render(self) -> String {
      let mut buffer = String::new();
      self.render_once(&mut buffer).unwrap();
      buffer
    }
  }

  #[test]
  pub fn test_from_string() {
    let markdown = "
 1. A list
 2. With some
 3. <span>Inline html</span>
    ";

    let buffer = Markdown::from_string(markdown).render();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li><span>Inline html</span></li>\n</ol>\n");
  }

  #[test]
  pub fn test_from_events() {
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

    let buffer = Markdown::from_events(events).render();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li>&lt;span&gt;Inline html&lt;/span&gt;</li>\n</ol>\n");
  }

  #[test]
  pub fn test_without_header_ids() {
    let markdown = "
# Header
## A Sub Header
    ";

    let buffer = Markdown::from_string(markdown).render();
    assert_eq!(buffer, "<h1>Header</h1>\n<h2>A Sub Header</h2>\n");
  }

  #[test]
  pub fn test_with_header_ids() {
    let markdown = "
# Header
## A Sub Header
    ";

    let buffer = Markdown::from_string(markdown).with_header_ids().render();
    assert_eq!(buffer, "<h1 id=\"header\">Header</h1>\n<h2 id=\"a-sub-header\">A Sub Header</h2>\n");
  }

  #[test]
  pub fn test_with_header_ids_and_linked_inline_image() {
    let markdown = "
# Header [![an image](http://example.com/image)](http://example.com)
    ";

    let buffer = Markdown::from_string(markdown).with_header_ids().render();
    assert_eq!(buffer, "<h1 id=\"header-an-image\">Header <a href=\"http://example.com\"><img src=\"http://example.com/image\" alt=\"an image\" /></a></h1>\n");
  }
}
