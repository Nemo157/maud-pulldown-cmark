use std::marker::PhantomData;

use pulldown_cmark as cmark;
use html_event as html;

use iter::{ Config, RootIter };

/// The adapter that allows rendering markdown inside a `maud` macro.
pub struct Markdown<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> {
  events: I,
  config: Config,
  phantom: PhantomData<&'a I>,
}

impl<'a> Markdown<'a, cmark::Parser<'a>> {
  /// To allow rendering from a string.
  pub fn from_string(s: &'a str) -> Markdown<'a, cmark::Parser<'a>> {
    Markdown {
      events: cmark::Parser::new(s),
      config: Config {
        header_ids: false,
      },
      phantom: PhantomData,
    }
  }
}

impl<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> Markdown<'a, I> {
  /// To allow rendering from a stream of events (useful for modifying the output of the general parser).
  pub fn from_events(events: I) -> Markdown<'a, I> {
    Markdown {
      events: events,
      config: Config {
        header_ids: false,
      },
      phantom: PhantomData,
    }
  }

  /// Generate ids for all headers, lowercases the text in the header and replaces spaces with -.
  pub fn with_header_ids(self) -> Markdown<'a, I> {
    Markdown {
      config: Config {
        header_ids: true,
        ..self.config
      },
      ..self
    }
  }
}

impl<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> IntoIterator for Markdown<'a, I> {
  type Item = Result<html::Event<'a>, ()>;
  type IntoIter = RootIter<'a, I>;
  fn into_iter(self) -> RootIter<'a, I> {
    RootIter::new(self.events, self.config)
  }
}

#[cfg(test)]
mod tests {
  use std::fmt::Write;
  use super::Markdown;
  use pulldown_cmark as cmark;

  impl<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> Markdown<'a, I> {
    pub fn render(self) -> String {
      self.into_iter().fold(String::new(), |mut result, event| { write!(result, "{}", event.unwrap()).unwrap(); result })
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

    let events = cmark::Parser::new(markdown).map(|ev| match ev {
      // Escape inline html
      cmark::Event::Html(html) | cmark::Event::InlineHtml(html) => cmark::Event::Text(html),
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
