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
