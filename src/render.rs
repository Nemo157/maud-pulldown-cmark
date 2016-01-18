
use std::borrow::{ Cow };
use std::collections::{ HashMap };
use std::fmt::{ self, Write };

use maud::{ self, Escaper };
use pulldown_cmark::{ Parser, Event, Tag };

use escape::HrefEscaper;
use markdown::Markdown;


impl<'a> maud::Render for Markdown<'a> {
  fn render(&self, w: &mut Write) -> fmt::Result {
    match self {
      &Markdown::FromString(s) => render_events(Parser::new(s), w),
    }
  }
}

fn render_events<'a, I: Iterator<Item=Event<'a>>>(events: I, mut w: &mut Write) -> fmt::Result {
  let mut numbers = HashMap::new();
  for event in events {
    try!(render_event(event, &mut numbers, &mut w));
  }
  Ok(())
}

fn render_event<'a>(event: Event<'a>, numbers: &mut HashMap<Cow<'a, str>, usize>, w: &mut Write) -> fmt::Result {
  match event {
    Event::Start(tag) => render_start_tag(tag, numbers, w),
    Event::End(tag) => render_end_tag(tag, w),
    Event::Text(text) => Escaper::new(w).write_str(&*text),
    Event::Html(html) | Event::InlineHtml(html) => w.write_str(&*html),
    Event::SoftBreak => w.write_char('\n'),
    Event::HardBreak => w.write_str("<br />\n"),
    Event::FootnoteReference(name) => render_footnote_reference(name, numbers, w),
  }
}

fn render_start_tag<'a>(tag: Tag<'a>, numbers: &mut HashMap<Cow<'a, str>, usize>, mut w: &mut Write) -> fmt::Result {
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

    Tag::Header(level) => render_header_start_tag(level as u8, w),
    Tag::CodeBlock(info) => render_code_block_start_tag(&*info, w),
    Tag::Image(src, title) => render_image_start_tag(&*src, &*title, w),
    Tag::Link(dest, title) => render_link_start_tag(&*dest, &*title, w),
    Tag::FootnoteDefinition(name) => render_footnote_definition_start_tag(name, numbers, w),
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
    Tag::Image(_, _) => Ok(()),
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

fn render_image_start_tag(src: &str, title: &str, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<img src=\""));
  try!(HrefEscaper::new(&mut w).write_str(src));
  try!(w.write_str("\" alt=\""));
  try!(Escaper::new(&mut w).write_str(title));
  w.write_str("\" />")
}

fn render_header_start_tag(level: u8, mut w: &mut Write) -> fmt::Result {
  try!(w.write_str("<h"));
  try!(w.write_char((b'0' + level) as char));
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

