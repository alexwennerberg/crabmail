// WIP
//
use crate::models::*;
use crate::templates::PAGE_SIZE;
use crate::time::Date;
use crate::util::*;
use nanotemplate::template;

impl Lists {
    pub fn to_gmi(&self) -> String {
        let mut lists = String::new();
        for list in &self.lists {
            lists.push_str(&format!("=> ./{0}/ {0}\n", &h(&list.config.name)));
        }
        // this looks stupid ok I know
        template(
            r#"# Mail Archives
{lists}
         "#,
            &[("lists", &lists)],
        )
        .unwrap()
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
                    "## {0}\n{1}\n=>mailto:{2} {2}\n",
                    self.config.name, self.config.description, self.config.email
                );
                for thread in thread_topics {
                    threads.push_str(
                        // TODO reuse with html templates?
                        &template(
                            r#"
=> threads/{path_id}.gmi {subject}
{preview}
{from} | {replies} replies | {date}
"#,
                            &[
                                (
                                    "path_id",
                                    &h(thread.message.pathescape_msg_id().to_str().unwrap()),
                                ),
                                ("subject", &h(&thread.message.subject)),
                                ("replies", &thread.reply_count.to_string()),
                                ("preview", &h(&thread.message.preview)),
                                ("date", &h(&Date::from(thread.last_reply).ymd())),
                                (
                                    "from",
                                    &h(&thread. // awkawrd
                                message.from
                                .name
                                .clone()
                                .unwrap_or(thread.message.from.address.clone())
                                .clone()),
                                ),
                            ],
                        )
                        .unwrap(),
                    );
                }
                if n + 1 <= page_count {
                    threads.push_str(&format!("\n=> index-{}.gmi next", n + 1));
                }
                threads
            })
            .collect()
    }
}

impl Thread {
    pub fn to_gmi(&self) -> String {
        let mut out = format!(
            r#"# {}
        "#,
            self.messages[0].subject.replace("\n", " ")
        );
        for msg in &self.messages {
            let mut optional_headers = String::new();
            if let Some(irt) = &msg.in_reply_to {
                optional_headers.push_str(&format!("\nIn-Reply-To: {}", &h(&irt)));
            }
            // TODO no copy pasta
            let cc_string = &h(&msg
                .cc
                .iter()
                .map(|t| t.to_string())
                .collect::<Vec<String>>()
                .join(", "));
            if msg.cc.len() > 0 {
                optional_headers.push_str(&format!("\nCc: {}", cc_string));
            }
            let body = match msg.flowed {
                true => unformat_flowed(&msg.body),
                false => msg.body.clone(),
            };
            let msg = template(
                r#"
## {subject}
From: {from}
Date: {date}
Message-Id: {msg_id}
To: {to}{optional_headers}
=> {mailto} Reply
=> ../messages/{msg_path}.mbox Export
--------------------------------------
{body}
"#,
                &[
                    ("subject", &h(&msg.subject)),
                    ("date", &h(&msg.date)),
                    ("msg_id", &h(&msg.id)),
                    (
                        "to",
                        &h(&msg
                            .to
                            .iter()
                            .map(|t| t.to_string())
                            .collect::<Vec<String>>()
                            .join(", ")),
                    ),
                    ("optional_headers", &optional_headers),
                    ("from", &h(&msg.from.address)),
                    ("mailto", &h(&msg.mailto)),
                    ("msg_path", &h(msg.pathescape_msg_id().to_str().unwrap())),
                    // TODO escape # in body?
                    // TODO only unformat flowed if flowed is true
                    ("body", &body),
                ],
            )
            .unwrap();
            out.push_str(&msg);
        }
        out
    }
}

// escape header
fn h(s: &str) -> String {
    s.replace("\n", " ")
}
