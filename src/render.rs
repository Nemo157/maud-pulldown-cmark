use std::ascii::AsciiExt;
use std::mem;
use std::borrow::{ Cow };
use std::collections::{ HashMap };
use std::fmt::Write;

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

pub fn render_events<'a, I: Iterator<Item=Event<'a>>>(config: Config, events: I, mut w: &mut String) {
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

fn render_events_internal<'a, I: Iterator<Item=Event<'a>>>(context: &mut Context<'a>, events: I, mut w: &mut String) {
  for event in events {
    if context.within_header {
      render_event_within_header(event, context, &mut w);
    } else if context.within_image {
      render_event_within_image(event, context, &mut w);
    } else {
      render_event(event, context, &mut w);
    }
  }
}

fn render_event<'a>(event: Event<'a>, context: &mut Context<'a>, w: &mut String) {
  match event {
    Event::Start(tag) => render_start_tag(tag, context, w),
    Event::End(tag) => render_end_tag(tag, w),
    Event::Text(text) => Escaper::new(w).write_str(&*text).unwrap(),
    Event::Html(html) | Event::InlineHtml(html) => w.push_str(&*html),
    Event::SoftBreak => w.push('\n'),
    Event::HardBreak => w.push_str("<br />\n"),
    Event::FootnoteReference(name) => render_footnote_reference(name, &mut context.numbers, w),
  }
}

fn render_event_within_image<'a>(event: Event<'a>, context: &mut Context<'a>, w: &mut String) {
  match event {
    Event::Start(_) => (),
    Event::End(Tag::Image(_, title)) => {
      context.within_image = false;
      render_image_end_tag(&*title, w);
    },
    Event::End(_) => (),
    Event::Text(text) => Escaper::new(w).write_str(&*text).unwrap(),
    Event::Html(_) => (),
    Event::InlineHtml(html) => Escaper::new(w).write_str(&*html).unwrap(),
    Event::SoftBreak | Event::HardBreak => w.push(' '),
    Event::FootnoteReference(name) => render_footnote_reference_within_image(name, &mut context.numbers, w),
  }
}

fn render_event_within_header<'a>(event: Event<'a>, context: &mut Context<'a>, w: &mut String) {
  match event {
    Event::Start(Tag::Header(_)) => { },
    Event::End(Tag::Header(level)) => {
      context.within_header = false;
      let id = mem::replace(&mut context.header_text, String::new());
      let events = mem::replace(&mut context.header_queue, Vec::new());
      render_header_start_tag(level as u8, Some(id), w);
      render_events_internal(context, events.into_iter(), w);
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
    },
    Event::SoftBreak | Event::HardBreak => {
      context.header_text.push('-');
      context.header_queue.push(event);
    },
    _ => {
      context.header_queue.push(event);
    },
  }
}

fn render_start_tag<'a>(tag: Tag<'a>, context: &mut Context<'a>, mut w: &mut String) {
  match tag {
    Tag::Rule => w.push_str("<hr />\n"),
    Tag::Code => w.push_str("<code>"),
    Tag::Strong => w.push_str("<strong>"),
    Tag::Emphasis => w.push_str("<em>"),
    Tag::Paragraph => w.push_str("<p>"),
    Tag::BlockQuote => w.push_str("<blockquote>\n"),

    Tag::Table(_) => w.push_str("<table>"),
    Tag::TableHead => w.push_str("<thead><tr>"),
    Tag::TableRow => w.push_str("<tr>"),
    Tag::TableCell => w.push_str("<td>"),

    Tag::Item => w.push_str("<li>"),
    Tag::List(None) => w.push_str("<ul>\n"),
    Tag::List(Some(1)) => w.push_str("<ol>\n"),
    Tag::List(Some(start)) => write!(w, "<ol start=\"{}\">\n", start).unwrap(),

    Tag::Header(level) => {
      if context.config.header_ids {
        context.within_header = true;
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

fn render_end_tag<'a>(tag: Tag<'a>, mut w: &mut String) {
  match tag {
    Tag::Rule => {},
    Tag::Code => w.push_str("</code>"),
    Tag::Strong => w.push_str("</strong>"),
    Tag::Emphasis => w.push_str("</em>"),
    Tag::Paragraph => w.push_str("</p>\n"),
    Tag::BlockQuote => w.push_str("</blockquote>\n"),

    Tag::Table(_) => w.push_str("</table>\n"),
    Tag::TableHead => w.push_str("</tr></thead>\n"),
    Tag::TableRow => w.push_str("</tr>\n"),
    Tag::TableCell => w.push_str("</td>"),

    Tag::Item => w.push_str("</li>\n"),
    Tag::List(None) => w.push_str("</ul>\n"),
    Tag::List(Some(_)) => w.push_str("</ol>\n"),

    Tag::Header(level) => render_header_end_tag(level as u8, w),
    Tag::CodeBlock(_) => w.push_str("</code></pre>\n"),
    Tag::Image(_, _) => (),
    Tag::Link(_, _) => w.push_str("</a>"),
    Tag::FootnoteDefinition(_) => w.push_str("</div>\n"),
  }
}

fn render_link_start_tag(dest: &str, title: &str, mut w: &mut String) {
  w.push_str("<a href=\"");
  HrefEscaper::new(&mut w).write_str(dest).unwrap();
  if !title.is_empty() {
    w.push_str("\" title=\"");
    Escaper::new(&mut w).write_str(title).unwrap();
  }
  w.push_str("\">")
}

fn render_image_start_tag(src: &str, mut w: &mut String) {
  w.push_str("<img src=\"");
  HrefEscaper::new(&mut w).write_str(src).unwrap();
  w.push_str("\" alt=\"")
}

fn render_image_end_tag(title: &str, mut w: &mut String) {
  if !title.is_empty() {
    w.push_str("\" title=\"");
    Escaper::new(&mut w).write_str(title).unwrap();
  }
  w.push_str("\" />")
}

fn render_header_start_tag(level: u8, id: Option<String>, mut w: &mut String) {
  w.push_str("<h");
  w.push((b'0' + level) as char);
  if let Some(id) = id {
    w.push_str(" id=\"");
    w.push_str(&*id);
    w.push('"');
  }
  w.push('>')
}

fn render_header_end_tag(level: u8, mut w: &mut String) {
  w.push_str("</h");
  w.push((b'0' + level) as char);
  w.push_str(">\n")
}

fn render_code_block_start_tag(info: &str, mut w: &mut String) {
  let lang = info.split(' ').next().unwrap();
  if lang.is_empty() {
    w.push_str("<pre><code>")
  } else {
    w.push_str("<pre><code class=\"language-");
    Escaper::new(&mut w).write_str(lang).unwrap();
    w.push_str("\">")
  }
}

fn render_footnote_definition_start_tag<'a>(name: Cow<'a, str>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut String) {
  w.push_str("<div class=\"footnote-definition\" id=\"");
  Escaper::new(&mut w).write_str(&*name).unwrap();
  let len = numbers.len();
  let number = numbers.entry(name).or_insert(len);
  write!(w, "\"><sup class=\"footnote-definition-label\">{}</sup>", number).unwrap();
}

fn render_footnote_reference<'a>(name: Cow<'a, str>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut String) {
  w.push_str("<sup class=\"footnote-reference\"><a href=\"");
  Escaper::new(&mut w).write_str(&*name).unwrap();
  let len = numbers.len();
  let number = numbers.entry(name).or_insert(len);
  write!(w, "\">{}</a></sup>", number).unwrap();
}

fn render_footnote_reference_within_image<'a>(name: Cow<'a, str>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut String) {
  let len = numbers.len();
  let number = numbers.entry(name).or_insert(len);
  write!(w, "[{}]", number).unwrap();
}

