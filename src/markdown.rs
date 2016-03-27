use std::fmt;
use std::marker::PhantomData;

use maud::RenderOnce;
use pulldown_cmark::{ Parser, Event };

use render;

/// The adapter that allows rendering markdown inside a `maud` macro.
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
