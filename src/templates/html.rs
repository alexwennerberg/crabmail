use super::util::xml_escape;
use super::util::xml_safe as x;
use crate::models::*;
use linkify::{LinkFinder, LinkKind};
use nanotemplate::template;
use std::borrow::Cow;

const header: &str = r#"<!DOCTYPE html>
<html>
<head>
<title>{title}</title>
<meta http-equiv='Permissions-Policy' content='interest-cohort=()'/>
<link rel='stylesheet' type='text/css' href='../../style.css' />
<meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0,user-scalable=0' />
<link rel='icon' href='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📧</text></svg>'></head>
<meta name="description" content="{title}"/>
<body>
"#;

const footer: &str = r#"
<hr>
Archive generated with  <a href='https://crabmail.flounder.online/'>crabmail</a>
</body>
</html>
"#;

impl Lists {
    pub fn to_html(&self) -> String {
        template(
            r#"
        <h1>Mail Archives</h1>
        <hr>
         "#,
            &[("title", "tbd")],
        )
        .unwrap()
    }
}

impl List {
    pub fn to_html(&self) -> Vec<String> {
        template(
            r#"
        <h1 class="page-title">
        {title}
        <a href="atom.xml"> {
            <img alt="Atom feed" src={rss_svg} />
        </a>
        </h1>
                 "#,
            &[("title", self.config.title.as_str()), ("rss_svg", RSS_SVG)],
        )
        .unwrap();
        vec![]
    }
}

impl Thread {
    pub fn to_html(&self) -> String {
        let root = &self.messages[0];
        let mut body = r#"
        <h1 class="page-title">{title}</h1>
        <a href="{path_id}.xml"><img alt="Atom Feed" src='{rss_svg}'></a>
        <div>
        <a href="../">Back</a>
        <a href="\#bottom">Latest</a>
        <hr>
        <div>
         "#
        .to_string();
        for msg in &self.messages {
            let ms = "newmail";
            body.push_str(ms);
        }
        body.push_str("</div>");
        template(
            &format!("{}{}{}", header, body, footer),
            // TODO html escape
            &[
                ("title", x(&root.subject).as_ref()),
                ("rss_svg", RSS_SVG),
                ("path_id", &x(root.pathescape_msg_id().to_str().unwrap())),
            ],
        )
        .unwrap()
    }
}

impl StrMessage {
    pub fn to_html(&self) -> String {
        // TODO test thoroughly
        template(
            r#"<div id="{id}", class="message">
               <div class="message-meta"> 
               <span class="bold">
               {subject}
               </span>
               <a href="mailto:{from}" class="bold">{from}</a>
               <span class="light">{date} 
               <a class="permalink" href=#{id}>🔗</a>
               </div>
               </div>
               etc
        "#,
            &[("id", "asdf")],
        )
        .unwrap()
    }
}

// gpl licensed from wikipedia https://commons.wikimedia.org/wiki/File:Generic_Feed-icon.svg
pub const RSS_SVG: &'static str = r#"
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

// TODO MOVE!
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
