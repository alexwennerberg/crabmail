use linkify::{LinkFinder, LinkKind};

// partly stolen from
// https://github.com/robinst/linkify/blob/demo/src/lib.rs#L5

pub fn email_body(body: &str) -> String {
    let mut bytes = Vec::new();
    let mut in_reply: bool = false;
    for line in body.lines() {
        if line.starts_with(">") || (line.starts_with("On ") && line.ends_with("wrote:")) {
            if !in_reply {
                in_reply = true;
                bytes.extend_from_slice(b"<span class='reply-text'>");
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

// less efficient, easier api
fn xml_safe(text: &str) -> String {
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
