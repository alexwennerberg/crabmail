// WIP
//
use crate::models::*;
use crate::templates::PAGE_SIZE;
use crate::util::*;
use std::fmt::Write;

macro_rules! lines {
    ($($line:literal),+ $(,)?) => {
        concat!(
            $($line, "\r\n"),+
        )
    };
}

impl Lists {
    pub fn to_gmi(&self) -> String {
        let mut lists = String::new();
        for list in &self.lists {
            // writing to a string so discarding errors is fine
            let _ = write!(lists, "=> ./{0}/ {0}\r\n", h(&list.config.name));
        }
        format!("# Mail Archives\r\n{lists}")
    }
}

impl List {
    pub fn to_gmi(&self) -> Vec<String> {
        // TODO paginate
        let page_count = self.thread_topics.len() / PAGE_SIZE + 1;
        self.thread_topics
            .chunks(PAGE_SIZE)
            .enumerate()
            .map(|(n, thread_topics)| {
                let mut threads = format!(
                    "## {0}\r\n{1}\r\n=>mailto:{2} {2}\r\n",
                    self.config.name, self.config.description, self.config.email
                );
                for thread in thread_topics {
                    // writing to a string so discarding errors is fine
                    let _ = write!(
                        threads,
                        lines!(
                            "=>threads/{path_id}.gmi {subject}",
                            "{preview}",
                            "{from} | {replies} replies | {date}",
                        ),
                        path_id = h(thread.message.pathescape_msg_id().to_str().unwrap()),
                        subject = h(&thread.message.subject),
                        replies = thread.reply_count,
                        preview = h(&thread.message.preview),
                        // get only year-month-day part of datetime
                        date = thread.last_reply.to_rfc3339().split_once('T').unwrap().0,
                        from = h(thread
                            .message
                            .from
                            .name
                            .as_ref()
                            .unwrap_or(&thread.message.from.address)),
                    );
                }
                if n < page_count {
                    // writing to a string so discarding errors is fine
                    let _ = write!(threads, "\r\n=> index-{}.gmi next", n + 1);
                }
                threads
            })
            .collect()
    }
}

impl Thread {
    pub fn to_gmi(&self) -> String {
        let mut out = format!("# {}\r\n", h(&self.messages[0].subject),);
        for msg in &self.messages {
            let mut optional_headers = String::new();
            if let Some(irt) = &msg.in_reply_to {
                // writing to a string so discarding errors is fine
                let _ = write!(optional_headers, "\r\nIn-Reply-To: {}", h(irt));
            }
            if !msg.cc.is_empty() {
                // writing to a string so discarding errors is fine
                let _ = write!(
                    optional_headers,
                    "\r\nCc: {}",
                    h(&msg
                        .cc
                        .iter()
                        .map(|t| t.to_string())
                        .collect::<Vec<String>>()
                        .join(", "))
                );
            }
            let body = match msg.flowed {
                true => unformat_flowed(&msg.body),
                false => msg.body.clone(),
            };
            // writing to a string so discarding errors is fine
            let _ = write!(
                out,
                lines!(
                    "## {subject}",
                    "From: {from}",
                    "Date: {date}",
                    "Message-Id: {msg_id}",
                    "To: {to}{optional_headers}",
                    "=> {mailto} Reply",
                    "=> ../messages/{msg_path}.mbox Export",
                    "--------------------------------------",
                    "{body}",
                ),
                subject = h(&msg.subject),
                date = msg.date,
                msg_id = h(&msg.id),
                to = h(&msg
                    .to
                    .iter()
                    .map(|t| t.to_string())
                    .collect::<Vec<String>>()
                    .join(", ")),
                optional_headers = optional_headers,
                from = h(&msg.from.address),
                mailto = h(&msg.mailto),
                msg_path = h(msg.pathescape_msg_id().to_str().unwrap()),
                // TODO escape # in body?
                // TODO only unformat flowed if flowed is true
                body = body,
            );
        }
        out
    }
}

// escape header
fn h(s: &str) -> String {
    s.replace(&['\r', '\n'], " ")
}
