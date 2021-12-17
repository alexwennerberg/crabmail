use linkify::{LinkFinder, LinkKind};
use std::time::{SystemTime, UNIX_EPOCH};

const SOLAR_YEAR_SECS: u64 = 31556926;
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
                    escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"\">");
                    escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"</a>");
                }
                Some(LinkKind::Email) => {
                    bytes.extend_from_slice(b"<a href=\"mailto:");
                    escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"\">");
                    escape(span.as_str(), &mut bytes);
                    bytes.extend_from_slice(b"</a>");
                }
                _ => {
                    escape(span.as_str(), &mut bytes);
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

fn escape(text: &str, dest: &mut Vec<u8>) {
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
pub fn timeago(unixtime: u64) -> String {
    let current_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    if unixtime > current_time {
        return "in the future".to_owned();
    }
    let diff = current_time - unixtime;
    let amount: u64;
    let metric: &str;
    if diff < 60 {
        amount = diff;
        metric = "second";
    } else if diff < 60 * 60 {
        amount = diff / 60;
        metric = "minute";
    } else if diff < 60 * 60 * 24 {
        amount = diff / (60 * 60);
        metric = "hour";
    } else if diff < SOLAR_YEAR_SECS * 2 {
        amount = diff / (60 * 60 * 24);
        metric = "day";
    } else {
        amount = diff / SOLAR_YEAR_SECS * 2;
        metric = "year";
    }
    match amount {
        1 => format!("{} {} ago", amount, metric),
        _ => format!("{} {}s ago", amount, metric),
    }
}
