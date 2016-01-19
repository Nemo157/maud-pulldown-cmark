/// The adapter that allows rendering markdown inside a `maud` macro.
///
/// # Examples
///
#[cfg_attr(feature = "nightly", doc = " ```")]
#[cfg_attr(not(feature = "nightly"), doc = " ```ignore")]
/// # #![feature(plugin)]
/// # #![plugin(maud_macros)]
/// # extern crate maud;
/// # extern crate maud_pulldown_cmark;
/// # use maud_pulldown_cmark::Markdown;
/// # fn main() {
/// let markdown = "
///  1. A list
///  2. With some
///  3. Values
/// ";
///
/// let mut buffer = String::new();
///
/// html!(buffer, {
///   div {
///     $(Markdown::FromString(markdown))
///   }
/// }).unwrap();
///
/// println!("{}", buffer);
/// # }
/// ```

pub enum Markdown<'a> {
  /// To allow rendering from a string.
  FromString(&'a str),
}

#[cfg(test)]
mod tests {
  #[test]
  pub fn test() {
    use maud::Render;
    use super::Markdown;

    let markdown = "
 1. A list
 2. With some
 3. Values
    ";

    let mut buffer = String::new();
    Markdown::FromString(markdown).render(&mut buffer).unwrap();
    assert_eq!(buffer, "<ol>\n<li>A list</li>\n<li>With some</li>\n<li>Values</li>\n</ol>\n");
  }
}
