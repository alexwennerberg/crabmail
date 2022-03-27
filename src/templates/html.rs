use super::util::xml_escape;
use super::util::xml_safe as x;
use crate::models::*;
use crate::templates::PAGE_SIZE;
use crate::time::Date;
use crate::util::*;
use linkify::{LinkFinder, LinkKind};
use nanotemplate::template;

const HEADER: &str = r#"<!DOCTYPE html>
<html>
<head>
<title>{title}</title>
<meta http-equiv='Permissions-Policy' content='interest-cohort=()'/>
<link rel='stylesheet' type='text/css' href='{css_path}' />
<meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0,user-scalable=0' />
<link rel='icon' href='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📧</text></svg>'>
<meta name="description" content="{title}"/>
</head>
<body>
"#;

const FOOTER: &str = r#"
Archive generated with  <a href='https://crabmail.flounder.online/'>crabmail</a>
</body>
</html>
"#;

impl Lists {
    pub fn to_html(&self) -> String {
        let mut lists = String::new();
        for list in &self.lists {
            lists.push_str(&format!(
                "<a href='./{0}'><h2>{0}</h2></a>\n",
                &x(&list.config.name)
            ));
        }
        let body = r#"<h1 class="page-title">{title}</h1>
            <hr>
            {lists}
            <hr>"#;
        template(
            &format!("{}{}{}", HEADER, body, FOOTER),
            &[
                ("title", "Mail Archives"),
                ("css_path", "style.css"),
                ("lists", &lists),
            ],
        )
        .unwrap()
    }
}

impl List {
    pub fn to_html(&self) -> Vec<String> {
        // TODO paginate
        let page_count = self.thread_topics.len() / PAGE_SIZE + 1;
        let mut page_idx = "<b>Pages</b>: ".to_string();
        for n in 0..page_count {
            let path = match n {
                0 => "index.html".to_string(),
                n => format!("index-{}.html", n + 1),
            };
            page_idx.push_str(&format!("<a href='{}'>{}</a> ", path, n + 1));
        }
        self.thread_topics
            .chunks(PAGE_SIZE)
            .enumerate()
            .map(|(n, thread_topics)| {
                let mut threads = String::new();
                for thread in thread_topics {
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
                    &format!(
                        "{}{}{}",
                        HEADER,
                        r#"
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
        {page_idx}
        <hr>
                 "#,
                        FOOTER
                    ),
                    &[
                        ("header", HEADER),
                        ("description", &self.config.description),
                        ("title", self.config.title.as_str()),
                        ("css_path", "../style.css"),
                        ("threads", &threads),
                        ("page_idx", &page_idx),
                        ("list_email", &self.config.email),
                        ("rss_svg", RSS_SVG),
                        ("footer", FOOTER),
                    ],
                )
                .unwrap();
                page
            })
            .collect()
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
            &format!("{}{}", HEADER, body),
            // TODO html escape
            &[
                ("title", x(&root.subject).as_ref()),
                ("css_path", "../../style.css"),
                ("rss_svg", RSS_SVG),
                ("path_id", &x(root.pathescape_msg_id().to_str().unwrap())),
            ],
        )
        .unwrap();
        for msg in &self.messages {
            // TODO converted from html
            // fix from header parsing
            // TODO in reply to
            let in_reply_to = if let Some(irt) = &msg.in_reply_to {
                format!(
                    "In-Reply-To: <a href='#{0}'>{1}</a><br>\n",
                    x(irt),
                    x(&truncate_ellipsis(irt, 40))
                )
            } else {
                String::new()
            };
            let mut extra_headers =
                format!("Message-Id: <a href='#{0}'>{0}</a><br>\n", &x(&msg.id));

            extra_headers.push_str("To: \n");
            extra_headers.push_str(
                &msg.to
                    .iter()
                    .map(|x| x.to_html())
                    .collect::<Vec<String>>()
                    .join(","),
            );
            // todo no copy pasta
            extra_headers.push_str("<br>\n");
            if msg.cc.len() > 0 {
                extra_headers.push_str("Cc: ");
                extra_headers.push_str(
                    &msg.cc
                        .iter()
                        .map(|x| x.to_html())
                        .collect::<Vec<String>>()
                        .join(","),
                );
                extra_headers.push_str("<br>\n");
            }

            let ms = r#"<div id="{msg_id}" class="message">
            <div class="message-meta">
            <span class="bold">
                {subject}
            </span>
            <br>
            From: {from}
            <br>
            Date: <span>{date}</span>
            <br>
            {in_reply_to}
            <details>
            <summary>More</summary>
            {extra_headers}
            </details>
            <a class="bold" href="{mailto}">Reply</a>
            [<a href="../messages/{msg_path}.eml">Export</a>]
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
                        ("msg_path", &x(msg.pathescape_msg_id().to_str().unwrap())),
                        ("subject", &x(&msg.subject)),
                        ("mailto", &x(&msg.mailto)),
                        ("from", &msg.from.to_html()),
                        ("date", &x(&msg.date)),
                        ("in_reply_to", &in_reply_to),
                        ("extra_headers", &extra_headers),
                        ("body", &email_body(&msg.body, msg.flowed)),
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
pub fn email_body(body: &str, flowed: bool) -> String {
    let mut bytes = Vec::new();
    let mut body = body;
    let tmp;
    if flowed {
        tmp = unformat_flowed(body);
        body = &tmp;
    }
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
