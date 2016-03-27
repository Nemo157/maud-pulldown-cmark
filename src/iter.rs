use std::ascii::AsciiExt;
use std::borrow::Cow;
use std::iter;
use std::marker::PhantomData;
use std::collections::HashMap;
use std::vec;

use pulldown_cmark as cmark;
use html_event as html;

type Events<'a> = Box<Iterator<Item=Result<html::Event<'a>, ()>> + 'a>;

static EMPTY_ATTRS: &'static [html::Attribute<'static>] = &[];

#[derive(Clone)]
pub struct Config {
  pub header_ids: bool,
}

/// The iterator
pub struct RootIter<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> {
  events: I,
  sub_iter: Option<Events<'a>>,
  config: Config,
  numbers: HashMap<Cow<'a, str>, usize>,
  phantom: PhantomData<&'a I>,
}

macro_rules! err {
  () => {
    (Box::new(Some(Err(())).into_iter()) as Events<'a>)
  }
}

macro_rules! events {
  () => {
    (Box::new(iter::empty()) as Events<'a>)
  };
  ($($e:expr,)*) => {
    (Box::new(vec![$(Ok($e),)*].into_iter()) as Events<'a>)
  };
  ($($e:expr),*) => {
    (events!($($e,)*))
  };
}

impl<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> RootIter<'a, I> {
  pub fn new(events: I, config: Config) -> RootIter<'a, I> {
    RootIter {
      events: events,
      sub_iter: Some(events![]),
      config: config,
      phantom: PhantomData,
      numbers: HashMap::new(),
    }
  }

  fn next_sub_iter(&mut self) -> Option<Events<'a>> {
    match self.events.next() {
      Some(cmark::Event::Start(tag)) =>
        Some(self.start_tag_to_events(tag)),
      Some(cmark::Event::End(tag)) =>
        Some(self.end_tag_to_events(tag)),
      Some(cmark::Event::Text(text)) =>
        Some(events![text!(text)]),
      Some(cmark::Event::Html(html)) | Some(cmark::Event::InlineHtml(html)) =>
        Some(events![raw_html!(html)]),
      Some(cmark::Event::SoftBreak) =>
        Some(events![text!("\n")]),
      Some(cmark::Event::HardBreak) =>
        Some(events![closed_tag!("br")]),
      Some(cmark::Event::FootnoteReference(name)) => {
        let len = self.numbers.len();
        let number = self.numbers.entry(name.clone()).or_insert(len);
        Some(events![
          start_tag!("sup", attrs!["class" => "footnote-reference"]),
          start_tag!("a", attrs!["href" => name]),
          text!(format!("{}", len)),
          end_tag!("a"),
          end_tag!("sup"),
        ])
      },
      None =>
        None,
    }
  }

  fn start_tag_to_events(&mut self, tag: cmark::Tag<'a>) -> Events<'a> {
    match tag {
      cmark::Tag::Rule => events![closed_tag!("hr")],
      cmark::Tag::Code => events![start_tag!("code")],
      cmark::Tag::Strong => events![start_tag!("strong")],
      cmark::Tag::Emphasis => events![start_tag!("em")],
      cmark::Tag::Paragraph => events![start_tag!("p")],
      cmark::Tag::BlockQuote => events![start_tag!("blockquote"), text!("\n")],
      cmark::Tag::Table(_) => events![start_tag!("table")],
      cmark::Tag::TableHead => events![start_tag!("thead"), start_tag!("tr")],
      cmark::Tag::TableRow => events![start_tag!("tr")],
      cmark::Tag::TableCell => events![start_tag!("td")],
      cmark::Tag::Item => events![start_tag!("li")],
      cmark::Tag::List(None) => events![start_tag!("ul"), text!("\n")],
      cmark::Tag::List(Some(1)) => events![start_tag!("ol"), text!("\n")],
      cmark::Tag::List(Some(start)) => events![
        start_tag!("ol", attrs!["start" => format!("{}", start)]),
        text!("\n"),
      ],
      cmark::Tag::Header(level) => {
        if self.config.header_ids {
          Box::new(HeaderWithIdsIter::new(level, &mut self.events, &self.config)) as Events<'a>
        } else {
          events![start_tag!(format!("h{}", level))]
        }
      },
      cmark::Tag::CodeBlock(info) => {
        let lang = info.split(' ').next().unwrap();
        if lang.is_empty() {
          events![start_tag!("pre"), start_tag!("code")]
        } else {
          events![
            start_tag!("pre"),
            start_tag!("code", attrs!["class" => format!("language-{}", lang)])
          ]
        }
      },
      cmark::Tag::Image(src, title) => {
        Box::new(ImageIter::new(src, title, &mut self.events, &mut self.numbers)) as Events<'a>
      },
      cmark::Tag::Link(dest, title) => {
        if title.is_empty() {
          events![start_tag!("a", attrs!["href" => dest])]
        } else {
          events![start_tag!("a", attrs!["href" => dest, "title" => title])]
        }
      },
      cmark::Tag::FootnoteDefinition(name) => {
        let len = self.numbers.len();
        let number = self.numbers.entry(name.clone()).or_insert(len);
        events![
          start_tag!("div", attrs!["class" => "footnote-definition", "id" => name]),
          start_tag!("sup", attrs!["class" => "footnote-definition-label"]),
          text!(format!("{}", number)),
          end_tag!("sup"),
        ]
      },
    }
  }

  fn end_tag_to_events(&mut self, tag: cmark::Tag<'a>) -> Events<'a> {
    match tag {
      cmark::Tag::Rule => events![],
      cmark::Tag::Code => events![end_tag!("code")],
      cmark::Tag::Strong => events![end_tag!("strong")],
      cmark::Tag::Emphasis => events![end_tag!("em")],
      cmark::Tag::Paragraph => events![end_tag!("p"), text!("\n")],
      cmark::Tag::BlockQuote => events![end_tag!("blockquote"), text!("\n")],
      cmark::Tag::Table(_) => events![end_tag!("table"), text!("\n")],
      cmark::Tag::TableHead => events![end_tag!("tr"), end_tag!("thead"), text!("\n")],
      cmark::Tag::TableRow => events![end_tag!("tr"), text!("\n")],
      cmark::Tag::TableCell => events![end_tag!("td")],
      cmark::Tag::Item => events![end_tag!("li"), text!("\n")],
      cmark::Tag::List(None) => events![end_tag!("ul"), text!("\n")],
      cmark::Tag::List(Some(_)) => events![end_tag!("ol"), text!("\n")],
      cmark::Tag::Header(level) => events![end_tag!(format!("h{}", level)), text!("\n")],
      cmark::Tag::CodeBlock(_) => events![end_tag!("code"), end_tag!("pre"), text!("\n")],
      cmark::Tag::Image(_, _) => err!(),
      cmark::Tag::Link(_, _) => events![end_tag!("a")],
      cmark::Tag::FootnoteDefinition(_) => events![end_tag!("div"), text!("\n")],
    }
  }
}

impl<'a, I: 'a + Iterator<Item=cmark::Event<'a>>> Iterator for RootIter<'a, I> {
  type Item = Result<html::Event<'a>, ()>;
  fn next(&mut self) -> Option<Result<html::Event<'a>, ()>> {
    if self.sub_iter.is_some() {
      if let Some(item) = { self.sub_iter.as_mut().unwrap().next() } {
        Some(item)
      } else {
        self.sub_iter = self.next_sub_iter();
        self.next()
      }
    } else {
      None
    }
  }
}

struct ImageIter<'a> {
  event: Option<Result<html::Event<'a>, ()>>,
}

impl<'a> ImageIter<'a> {
  pub fn new(src: Cow<'a, str>, title: Cow<'a, str>, events: &mut Iterator<Item=cmark::Event<'a>>, numbers: &mut HashMap<Cow<'a, str>, usize>) -> ImageIter<'a> {
    let mut err = false;
    let mut alt = String::new();
    for event in events {
      match event {
        cmark::Event::Start(_) => { err = true; break; },
        cmark::Event::End(cmark::Tag::Image(_, _)) => {
          break;
        },
        cmark::Event::End(_) => { err = true; break; },
        cmark::Event::Text(text) => { alt.push_str(&*text) },
        cmark::Event::Html(_) => { err = true; break; },
        cmark::Event::InlineHtml(html) => { alt.push_str(&*html) },
        cmark::Event::SoftBreak | cmark::Event::HardBreak => { alt.push(' ') },
        cmark::Event::FootnoteReference(name) => {
          use std::fmt::Write;
          let len = numbers.len();
          let number = numbers.entry(name).or_insert(len);
          write!(alt, "[{}]", number).unwrap()
        }
      }
    }
    ImageIter {
      event: Some(if err {
        Err(())
      } else {
        if title.is_empty() {
          Ok(closed_tag!("img", attrs!["src" => src, "alt" => alt]))
        } else {
          Ok(closed_tag!("img", attrs!["src" => src, "title" => title, "alt" => alt]))
        }
      }),
    }
  }
}

impl<'a> Iterator for ImageIter<'a> {
  type Item = Result<html::Event<'a>, ()>;
  fn next(&mut self) -> Option<Result<html::Event<'a>, ()>> {
    self.event.take()
  }
}

struct HeaderWithIdsIter<'a> {
  id: Option<String>,
  err: bool,
  level: i32,
  started: bool,
  inner: RootIter<'a, vec::IntoIter<cmark::Event<'a>>>,
}

impl<'a> HeaderWithIdsIter<'a> {
  pub fn new(level: i32, events: &mut Iterator<Item=cmark::Event<'a>>, config: &Config) -> HeaderWithIdsIter<'a> {
    let mut id = String::new();
    let mut err = false;
    let mut result = Vec::new();
    for event in events {
      match event {
        cmark::Event::Start(cmark::Tag::Header(_)) => {
          err = true;
          break;
        },
        cmark::Event::End(cmark::Tag::Header(_)) => {
          result.push(event);
          break;
        },
        cmark::Event::Text(ref text) => {
          let t: String = text.chars()
            .map(|c| if c.is_whitespace() || !c.is_ascii() { '-' } else { c.to_ascii_lowercase() })
            .collect();
          id.push_str(&*t);
        },
        cmark::Event::SoftBreak | cmark::Event::HardBreak => {
          id.push('-');
        },
        cmark::Event::Start(cmark::Tag::FootnoteDefinition(_)) => {
          // TODO: support footnotes
          err = true;
          break;
        },
        cmark::Event::FootnoteReference(_) => {
          // TODO: support footnotes
          err = true;
          break;
        },
        _ => (),
      }
      result.push(event);
    }
    HeaderWithIdsIter {
      id: Some(id),
      err: err,
      level: level,
      started: false,
      inner: RootIter::new(result.into_iter(), config.clone()),
    }
  }
}

impl<'a> Iterator for HeaderWithIdsIter<'a> {
  type Item = Result<html::Event<'a>, ()>;
  fn next(&mut self) -> Option<Result<html::Event<'a>, ()>> {
    if self.started {
      self.inner.next()
    } else {
      self.started = true;
      Some(Ok(start_tag!(format!("h{}", self.level), attrs!["id" => self.id.take().unwrap()])))
    }
  }
}
