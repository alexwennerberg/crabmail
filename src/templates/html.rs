use super::util::xml_escape;
use super::util::xml_safe as x;
use crate::models::*;
use crate::time::Date;
use linkify::{LinkFinder, LinkKind};
use nanotemplate::template;
use std::borrow::Cow;

const header: &str = r#"<!DOCTYPE html>
<html>
<head>
<title>{title}</title>
<meta http-equiv='Permissions-Policy' content='interest-cohort=()'/>
<link rel='stylesheet' type='text/css' href='../style.css' />
<meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0,user-scalable=0' />
<link rel='icon' href='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📧</text></svg>'></head>
<meta name="description" content="{title}"/>
</head>
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
        let body = r#"<h1 class="page-title">{title}</h1>
            <a href="atom.xml"><img alt="Atom Feed" src='{rss_svg}' /></a>"#;
        template(
            &format!("{}{}{}", header, body, footer),
            &[("rss_svg", RSS_SVG), ("title", "tbd")],
        )
        .unwrap()
    }
}

impl List {
    pub fn to_html(&self) -> Vec<String> {
        // TODO paginate
        let mut threads = String::new();
        for thread in &self.thread_topics {
            threads.push_str(
                &template(
                    r#"
            <div class='message-sum'>
            <a class="bigger" href="threads/{path_id}.html">{subject}</a>
            <br>
            <div class="monospace">{preview}</div>
            <b>{from}</b> | {replies} replies | {date}<hr>
            "#,
                    &[
                        (
                            "path_id",
                            &x(thread.message.pathescape_msg_id().to_str().unwrap()),
                        ),
                        ("subject", &x(&thread.message.subject)),
                        ("replies", &thread.reply_count.to_string()),
                        ("date", &x(&Date::from(thread.last_reply).ymd())),
                        (
                            "from",
                            &x(&thread. // awkawrd
                                message.from
                                .name
                                .clone()
                                .unwrap_or(thread.message.from.address.clone())
                                .clone()),
                        ),
                        ("preview", &x(&thread.message.preview)),
                    ],
                )
                .unwrap(),
            );
        }
        // TODO use summary??
        let page = template(
            r#"
            {header}
        <h1 class="page-title">
        {title}
        <a href="atom.xml"> 
            <img alt="Atom feed" src='{rss_svg}' />
        </a>
        </h1>
        {description}<br>
        <a href="{mailto:list_email}">{list_email}</a>
        <hr>
        {threads}
        {footer}
                 "#,
            &[
                ("header", header),
                ("description", &self.config.description),
                ("title", self.config.title.as_str()),
                ("threads", &threads),
                ("list_email", &self.config.email),
                ("rss_svg", RSS_SVG),
                ("footer", footer),
            ],
        )
        .unwrap();
        vec![page]
    }
}

impl MailAddress {
    fn to_html(&self) -> String {
        let mut out = String::new();
        if let Some(n) = &self.name {
            out.push('"');
            out.push_str(&x(n));
            out.push_str("\" ");
        }
        out.push_str(&format!(
            "<<a href='mailto:{0}'>{0}</a>>",
            &x(&self.address)
        ));
        out
    }
}
impl Thread {
    pub fn to_html(&self) -> String {
        let root = &self.messages[0];
        let body = r#"
        <h1 class="page-title">{title}</h1>
        <a href="{path_id}.xml"><img alt="Atom Feed" src='{rss_svg}'></a>
        <div>
        <a href="../">Back</a>
        <a href='#bottom'>Latest</a>
        <hr>
        <div>
         "#;
        let mut out = template(
            &format!("{}{}", header, body),
            // TODO html escape
            &[
                ("title", x(&root.subject).as_ref()),
                ("rss_svg", RSS_SVG),
                ("path_id", &x(root.pathescape_msg_id().to_str().unwrap())),
            ],
        )
        .unwrap();
        for msg in &self.messages {
            // TODO converted from html
            // fix from header parsing
            // TODO in reply to
            let mut extra_headers = format!("Message-Id: {}<br>\n", &x(&msg.id));
            if let Some(irt) = &msg.in_reply_to {
                extra_headers.push_str(&format!("In-Reply-To: {}<br>\n", x(irt)));
            }
            extra_headers.push_str("To: ");
            // extra_headers.push_str(msg.to.iter().map(|x| x.to_html()).collect().join(","));
            extra_headers.push_str("Cc: \n");
            // extra_headers.push_str(msg.cc.iter().map(|x| x.to_html()).collect().join(","));

            extra_headers.push_str(&format!("Content-Type: {}<br>\n", &x(&msg.content_type)));
            // Content-Type: tbd <br>
            let ms = r#"<div id="{msg_id}" class="message">
            <div class="message-meta">
            <span class="bold">
                {subject}
            </span>
            <br>
            From: <a href="{from_addr}">"{from_name}" {from_addr}</a>
            <br>
            Date: <span class="light">{date}</span>
            <details>
            <summary>More headers</summary>
            {extra_headers}
            </details>
            <a class="bold" href="tbd">Reply</a>
            [<a href="tbd.eml">Download</a>]
            </div>
            <div class="email-body">
             {body}
            </div>
            </div>
            "#;
            out.push_str(
                &template(
                    ms,
                    &[
                        ("msg_id", &x(&msg.id)),
                        ("subject", &x(&msg.subject)),
                        ("from_addr", &x(&msg.from.address)),
                        ("from_name", &msg.from.to_html()),
                        ("date", &x(&msg.date)),
                        ("extra_headers", &extra_headers),
                        ("body", &email_body(&msg.body)),
                    ],
                )
                .unwrap(),
            );
        }
        out.push_str("</div><hr></body></html>");
        // body
        out
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
