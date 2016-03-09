use std::ascii::AsciiExt;
use std::mem;
use std::borrow::{ Cow };
use std::collections::{ HashMap };
use std::fmt::{ self, Write };

use maud::{ Escaper };
use pulldown_cmark::{ Event, Tag };

use escape::HrefEscaper;

pub struct Config {
  pub header_ids: bool,
}

struct Context<'a> {
  numbers: HashMap<Cow<'a, str>, usize>,
  within_image: bool,
  within_header: bool,
  header_queue: Vec<Event<'a>>,
  header_text: String,
  config: Config,
}

pub fn render_events<'a, I: Iterator<Item=Event<'a>>>(config: Config, events: I, mut w: &mut Write) -> fmt::Result {
  let mut context = Context {
    numbers: HashMap::new(),
    within_image: false,
    within_header: false,
    header_queue: Vec::new(),
    header_text: String::new(),
    config: config,
  };
  render_events_internal(&mut context, events, w)
}

fn render_events_internal<'a, I: Iterator<Item=Event<'a>>>(context: &mut Context<'a>, events: I, mut w: &mut Write) -> fmt::Result {
  for event in events {
    if context.within_header {
      try!(render_event_within_header(event, context, &mut w));
    } else if context.within_image {
      try!(render_event_within_image(event, context, &mut w));
    } else {
      try!(render_event(event, context, &mut w));
    }
  }
  Ok(())
}

fn render_event<'a>(event: Event<'a>, context: &mut Context<'a>, w: &mut Write) -> fmt::Result {
  match event {
    Event::Start(tag) => render_start_tag(tag, context, w),
    Event::End(tag) => render_end_tag(tag, w),
    Event::Text(text) => Escaper::new(w).write_str(&*text),
    Event::Html(html) | Event::InlineHtml(html) => w.write_str(&*html),
    Event::SoftBreak => w.write_char('\n'),
    Event::HardBreak => w.write_str("<br />\n"),
    Event::FootnoteReference(name) => render_footnote_reference(name, &mut context.numbers, w),
  }
}

fn render_event_within_image<'a>(event: Event<'a>, context: &mut Context<'a>, w: &mut Write) -> fmt::Result {
  match event {
    Event::Start(_) => Err(fmt::Error),
    Event::End(Tag::Image(_, title)) => {
      context.within_image = false;
      render_image_end_tag(&*title, w)
    },
    Event::End(_) => Err(fmt::Error),
    Event::Text(text) => Escaper::new(w).write_str(&*text),
    Event::Html(_) => Err(fmt::Error),
    Event::InlineHtml(html) => Escaper::new(w).write_str(&*html),
    Event::SoftBreak | Event::HardBreak => w.write_char(' '),
    Event::FootnoteReference(name) => render_footnote_reference_within_image(name, &mut context.numbers, w),
  }
}

fn render_event_within_header<'a>(event: Event<'a>, context: &mut Context<'a>, w: &mut Write) -> fmt::Result {
  match event {
    Event::Start(Tag::Header(_)) => {
      Err(fmt::Error)
    },
    Event::End(Tag::Header(level)) => {
      context.within_header = false;
      let id = mem::replace(&mut context.header_text, String::new());
      let events = mem::replace(&mut context.header_queue, Vec::new());
      try!(render_header_start_tag(level as u8, Some(id), w));
      try!(render_events_internal(context, events.into_iter(), w));
      render_header_end_tag(level as u8, w)
    },
    Event::Text(text) => {
      // TODO: WHY RUST WHY?
      // error: mismatched types:
      //  expected `&str`,
      //  found `[closure@src/render.rs:93:28: 93:72]`
      // let t = text.replace(|c: char| c.is_whitespace() || !c.is_ascii(), "-");
      let t: String = text.chars()
        .map(|c| if c.is_whitespace() || !c.is_ascii() { '-' } else { c.to_ascii_lowercase() })
        .collect();
      context.header_text.push_str(&*t);
      context.header_queue.push(Event::Text(text));
      Ok(())
    },
    Event::SoftBreak | Event::HardBreak => {
      context.header_text.push('-');
      context.header_queue.push(event);
      Ok(())
    },
    _ => {
      context.header_queue.push(event);
      Ok(())
    },
  }
}

fn render_start_tag<'a>(tag: Tag<'a>, context: &mut Context<'a>, mut w: &mut Write) -> fmt::Result {
  match tag {
    Tag::Rule => w.write_str("<hr />\n"),
    Tag::Code => w.write_str("<code>"),
    Tag::Strong => w.write_str("<strong>"),
    Tag::Emphasis => w.write_str("<em>"),
    Tag::Paragraph => w.write_str("<p>"),
    Tag::BlockQuote => w.write_str("<blockquote>\n"),

    Tag::Table(_) => w.write_str("<table>"),
    Tag::TableHead => w.write_str("<thead><tr>"),
    Tag::TableRow => w.write_str("<tr>"),
    Tag::TableCell => w.write_str("<td>"),

    Tag::Item => w.write_str("<li>"),
    Tag::List(None) => w.write_str("<ul>\n"),
    Tag::List(Some(1)) => w.write_str("<ol>\n"),
    Tag::List(Some(start)) => write!(w, "<ol start=\"{}\">\n", start),

    Tag::Header(level) => {
      if context.config.header_ids {
        context.within_header = true;
        Ok(())
      } else {
        render_header_start_tag(level as u8, None, w)
      }
    },
    Tag::CodeBlock(info) => render_code_block_start_tag(&*info, w),
    Tag::Image(src, _) => {
      context.within_image = true;
      render_image_start_tag(&*src, w)
    },
    Tag::Link(dest, title) => render_link_start_tag(&*dest, &*title, w),
    Tag::FootnoteDefinition(name) => render_footnote_definition_start_tag(name, &mut context.numbers, w),
  }
}

fn render_end_tag<'a>(tag: Tag<'a>, mut w: &mut Write) -> fmt::Result {
  match tag {
    Tag::Rule => Ok(()),
    Tag::Code => w.write_str("</code>"),
    Tag::Strong => w.write_str("</strong>"),
    Tag::Emphasis => w.write_str("</em>"),
    Tag::Paragraph => w.write_str("</p>\n"),
    Tag::BlockQuote => w.write_str("</blockquote>\n"),

    Tag::Table(_) => w.write_str("</table>\n"),
    Tag::TableHead => w.write_str("</tr></thead>\n"),
    Tag::TableRow => w.write_str("</tr>\n"),
    Tag::TableCell => w.write_str("</td>"),

    Tag::Item => w.write_str("</li>\n"),
    Tag::List(None) => w.write_str("</ul>\n"),
    Tag::List(Some(_)) => w.write_str("</ol>\n"),

    Tag::Header(level) => render_header_end_tag(level as u8, w),
    Tag::CodeBlock(_) => w.write_str("</code></pre>\n"),
    Tag::Image(_, _) => Err(fmt::Error),
    Tag::Link(_, _) => w.write_str("</a>"),
    Tag::FootnoteDefinition(_) => w.write_str("</div>\n"),
  }
}

fn render_link_start_tag(dest: &str, title: &str, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<a href=\""));
  try!(HrefEscaper::new(&mut w).write_str(dest));
  if !title.is_empty() {
    try!(w.write_str("\" title=\""));
    try!(Escaper::new(&mut w).write_str(title));
  }
  w.write_str("\">")
}

fn render_image_start_tag(src: &str, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<img src=\""));
  try!(HrefEscaper::new(&mut w).write_str(src));
  w.write_str("\" alt=\"")
}

fn render_image_end_tag(title: &str, mut w: &mut Write) -> fmt::Result {
  if !title.is_empty() {
    try!(w.write_str("\" title=\""));
    try!(Escaper::new(&mut w).write_str(title));
  }
  w.write_str("\" />")
}

fn render_header_start_tag(level: u8, id: Option<String>, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<h"));
  try!(w.write_char((b'0' + level) as char));
  if let Some(id) = id {
    try!(w.write_str(" id=\""));
    try!(w.write_str(&*id));
    try!(w.write_char('"'));
  }
  w.write_char('>')
}

fn render_header_end_tag(level: u8, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("</h"));
  try!(w.write_char((b'0' + level) as char));
  w.write_str(">\n")
}

fn render_code_block_start_tag(info: &str, mut w: &mut Write) -> fmt::Result {
  let lang = info.split(' ').next().unwrap();
  if lang.is_empty() {
    w.write_str("<pre><code>")
  } else {
    try!(w.write_str("<pre><code class=\"language-"));
    try!(Escaper::new(&mut w).write_str(lang));
    w.write_str("\">")
  }
}

fn render_footnote_definition_start_tag<'a>(name: Cow<'a, str>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<div class=\"footnote-definition\" id=\""));
  try!(Escaper::new(&mut w).write_str(&*name));
  let len = numbers.len();
  let number = numbers.entry(name).or_insert(len);
  write!(w, "\"><sup class=\"footnote-definition-label\">{}</sup>", number)
}

fn render_footnote_reference<'a>(name: Cow<'a, str>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<sup class=\"footnote-reference\"><a href=\""));
  try!(Escaper::new(&mut w).write_str(&*name));
  let len = numbers.len();
  let number = numbers.entry(name).or_insert(len);
  write!(w, "\">{}</a></sup>", number)
}

fn render_footnote_reference_within_image<'a>(name: Cow<'a, str>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut Write) -> fmt::Result {
  let len = numbers.len();
  let number = numbers.entry(name).or_insert(len);
  write!(w, "[{}]", number)
}

