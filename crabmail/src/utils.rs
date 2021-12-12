use std::fmt;
use std::io;
use std::io::Write;

// Derived from https://github.com/raphlinus/pulldown-cmark/blob/master/src/escape.rs
// Don't use single quotes (') in any of your attributes

const fn create_html_escape_table() -> [u8; 256] {
    let mut table = [0; 256];
    table[b'"' as usize] = 1;
    table[b'&' as usize] = 2;
    table[b'<' as usize] = 3;
    table[b'>' as usize] = 4;
    table
}

static HTML_ESCAPE_TABLE: [u8; 256] = create_html_escape_table();

static HTML_ESCAPES: [&str; 5] = ["", "&quot;", "&amp;", "&lt;", "&gt;"];

#[derive(Debug)]
pub struct EscapedHTML<'a>(pub &'a str);

impl fmt::Display for EscapedHTML<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0;
        let bytes = s.as_bytes();
        let mut mark = 0;
        let mut i = 0;
        // does this work
        while i < s.len() {
            match bytes[i..]
                .iter()
                .position(|&c| HTML_ESCAPE_TABLE[c as usize] != 0)
            {
                Some(pos) => {
                    i += pos;
                }
                None => break,
            }
            let c = bytes[i];
            let escape = HTML_ESCAPE_TABLE[c as usize];
            let escape_seq = HTML_ESCAPES[escape as usize];
            f.write_str(&s[mark..i])?;
            f.write_str(escape_seq)?;
            i += 1;
            mark = i; // all escaped characters are ASCII
        }
        f.write_str(&s[mark..])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        assert_eq!(
            format!("{}", EscapedHTML("<b>'hello&world\"</b>")),
            "&lt;b&gt;'hello&amp;world&quot;&lt;/b&gt;".to_string()
        );
    }
}
