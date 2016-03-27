use std::fmt::{ self, Write };

pub struct HrefEscaper<W: Write> {
  inner: W,
}

impl<W: Write> HrefEscaper<W> {
  pub fn new(inner: W) -> HrefEscaper<W> {
    HrefEscaper {
      inner: inner,
    }
  }
}

fn char_to_string(c: char) -> String {
  let mut s = String::with_capacity(4);
  s.push(c);
  s
}

fn percent_encode(c: char) -> String {
  let mut s = String::with_capacity(9);
  for b in char_to_string(c).as_bytes() {
    write!(s, "%{0:X}", b).unwrap();
  }
  s
}

// See https://url.spec.whatwg.org/ for escaping rules
fn escape_char(c: char) -> String {
  if c < '!' {
    // Control characters and space
    return percent_encode(c);
  }

  if c <= '\x7F' {
    // Disallowed ASCII characters
    return match c {
      '"' | '%' | '<' | '>' | '[' | '\\' => percent_encode(c),
      ']' | '^' | '`' | '{' | '|' | '}' | '\x7F' => percent_encode(c),
      _ => char_to_string(c),
    };
  }

  match c {
    // Allowed Unicode ranges
    '\u{00A0}'...'\u{D7FF}'
      | '\u{E000}'...'\u{FDCF}'
      | '\u{FDF0}'...'\u{FFFD}'
      | '\u{10000}'...'\u{1FFFD}'
      | '\u{20000}'...'\u{2FFFD}'
      | '\u{30000}'...'\u{3FFFD}'
      | '\u{40000}'...'\u{4FFFD}'
      | '\u{50000}'...'\u{5FFFD}'
      | '\u{60000}'...'\u{6FFFD}'
      | '\u{70000}'...'\u{7FFFD}'
      | '\u{80000}'...'\u{8FFFD}'
      | '\u{90000}'...'\u{9FFFD}'
      | '\u{A0000}'...'\u{AFFFD}'
      | '\u{B0000}'...'\u{BFFFD}'
      | '\u{C0000}'...'\u{CFFFD}'
      | '\u{D0000}'...'\u{DFFFD}'
      | '\u{E0000}'...'\u{EFFFD}'
      | '\u{F0000}'...'\u{FFFFD}'
      | '\u{100000}'...'\u{10FFFD}' => char_to_string(c),
    _ => percent_encode(c),
  }
}

impl<W: Write> Write for HrefEscaper<W> {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for c in s.chars() {
      try!(self.write_char(c));
    }
    Ok(())
  }

  fn write_char(&mut self, c: char) -> fmt::Result {
    self.inner.write_str(&*escape_char(c))
  }
}

#[cfg(test)]
mod tests {
  use super::HrefEscaper;

  macro_rules! test_url {
    ($url:expr, $expected:expr) => ({
      use std::fmt::Write;
      let mut s = String::with_capacity(31);
      HrefEscaper::new(&mut s).write_str($url).unwrap();
      assert_eq!(&*s, $expected);
    })
  }

  #[test]
  fn test_poop() {
    test_url!("https://example.org/💩", "https://example.org/💩");
  }

  #[test]
  fn test_ascii() {
    test_url!("https://example.org/something", "https://example.org/something");
  }

  #[test]
  fn test_ascii_escapes() {
    test_url!("https://example.org/\"", "https://example.org/%22");
    test_url!("https://example.org/%", "https://example.org/%25");
    test_url!("https://example.org/<", "https://example.org/%3C");
    test_url!("https://example.org/>", "https://example.org/%3E");
    test_url!("https://example.org/[", "https://example.org/%5B");
    test_url!("https://example.org/\\", "https://example.org/%5C");
    test_url!("https://example.org/]", "https://example.org/%5D");
    test_url!("https://example.org/^", "https://example.org/%5E");
    test_url!("https://example.org/`", "https://example.org/%60");
    test_url!("https://example.org/{", "https://example.org/%7B");
    test_url!("https://example.org/|", "https://example.org/%7C");
    test_url!("https://example.org/}", "https://example.org/%7D");
  }

  #[test]
  fn test_unicode_escapes() {
    test_url!("https://example.org/\u{92}", "https://example.org/%C2%92");
    test_url!("https://example.org/\u{FFFFE}", "https://example.org/%F3%BF%BF%BE");
  }
}
