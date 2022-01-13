use linkify::{LinkFinder, LinkKind};

// gpl licensed from wikipedia https://commons.wikimedia.org/wiki/File:Generic_Feed-icon.svg
pub const RSS_SVG: &str = r#"
data:image/svg+xml,<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg"
     id="RSSicon"
     viewBox="0 0 8 8" width="16" height="16">
  <title>RSS feed icon</title>
  <style type="text/css">
    .button {stroke: none; fill: orange;}
    .symbol {stroke: none; fill: white;}
  </style>
  <rect   class="button" width="8" height="8" rx="1.5" />
  <circle class="symbol" cx="2" cy="6" r="1" />
  <path   class="symbol" d="m 1,4 a 3,3 0 0 1 3,3 h 1 a 4,4 0 0 0 -4,-4 z" />
  <path   class="symbol" d="m 1,2 a 5,5 0 0 1 5,5 h 1 a 6,6 0 0 0 -6,-6 z" />
</svg>"#;

// partly stolen from
// https://github.com/robinst/linkify/blob/demo/src/lib.rs#L5
// Dual licensed under MIT and Apache
pub fn email_body(body: &str) -> String {
    let mut bytes = Vec::new();
    let mut in_reply: bool = false;
    for line in body.lines() {
        if line.starts_with(">") || (line.starts_with("On ") && line.ends_with("wrote:")) {
            if !in_reply {
                in_reply = true;
                bytes.extend_from_slice(b"<span class='light'>");
            }
        } else if in_reply {
            bytes.extend_from_slice(b"</span>");
            in_reply = false
        }

        let finder = LinkFinder::new();
        for span in finder.spans(line) {
            match span.kind() {
                Some(LinkKind::Url) => {
                    bytes.extend_from_slice(b"<a href=\"");
                    xml_escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"\">");
                    xml_escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"</a>");
                }
                Some(LinkKind::Email) => {
                    bytes.extend_from_slice(b"<a href=\"mailto:");
                    xml_escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"\">");
                    xml_escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"</a>");
                }
                _ => {
                    xml_escape(span.as_str(), &mut bytes);
                }
            }
        }
        bytes.extend(b"\n");
    }
    if in_reply {
        bytes.extend_from_slice(b"</span>");
    }
    // TODO err conversion
    String::from_utf8(bytes).unwrap()
}

// stolen from https://github.com/deltachat/deltachat-core-rust/blob/master/src/format_flowed.rs
// undoes format=flowed
pub fn unformat_flowed(text: &str) -> String {
    let mut result = String::new();
    let mut skip_newline = true;

    for line in text.split('\n') {
        // Revert space-stuffing
        let line = line.strip_prefix(' ').unwrap_or(line);

        if !skip_newline {
            result.push('\n');
        }

        if let Some(line) = line.strip_suffix(' ') {
            // Flowed line
            result += line;
            result.push(' ');
            skip_newline = true;
        } else {
            // Fixed line
            result += line;
            skip_newline = false;
        }
    }
    result
}

// less efficient, easier api
pub fn xml_safe(text: &str) -> String {
    // note we escape more than we need to
    let mut dest = Vec::new();
    xml_escape(text, &mut dest);
    std::str::from_utf8(&dest).unwrap().to_owned()
}

fn xml_escape(text: &str, dest: &mut Vec<u8>) {
    for c in text.bytes() {
        match c {
            b'&' => dest.extend_from_slice(b"&amp;"),
            b'<' => dest.extend_from_slice(b"&lt;"),
            b'>' => dest.extend_from_slice(b"&gt;"),
            b'"' => dest.extend_from_slice(b"&quot;"),
            b'\'' => dest.extend_from_slice(b"&#39;"),
            _ => dest.push(c),
        }
    }
}
