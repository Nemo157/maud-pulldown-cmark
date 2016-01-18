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

static HREF_SAFE: [bool; 0x80] = [
  false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
  false, false, false, false, false, false, false, false, false, false, false, false, false, false, false, false,
  false,  true, false,  true,  true,  true, false, false,  true,  true,  true,  true,  true,  true,  true,  true,
   true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true, false,  true, false,  true,
   true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
   true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true, false, false, false, false,  true,
  false,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true,
   true,  true,  true,  true,  true,  true,  true,  true,  true,  true,  true, false, false, false, false, false,
];

impl<W: Write> Write for HrefEscaper<W> {
  fn write_str(&mut self, s: &str) -> fmt::Result {
    for c in s.chars() {
      match c {
        '&' => {
          try!(self.inner.write_str("&amp;"));
        },
        '\'' => {
          try!(self.inner.write_str("&#x27;"));
        },
        c if c > '\x7F' || !HREF_SAFE[c as usize] => {
          unimplemented!(); // Need to convert to hex escape string
        },
        _ => {
          try!(self.inner.write_char(c));
        },
      }
    }
    Ok(())
  }
}
