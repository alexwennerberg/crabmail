use super::util::xml_safe as x;
use crate::models::*;
use crate::time::Date;
use crate::util::unformat_flowed;
// use crate::templates::util::xml_safe;
// use anyhow::{Context, Result};
use nanotemplate::template;

const FEED_TEMPLATE: &str = r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>{feed_title}</title>
<link href="{feed_link}"/>
<updated>{last_updated}</updated>
<author>
<name>{author_name}</name>
<email>{author_email}</email>
</author>
<id>{feed_id}</id>
{entry_list}
</feed>"#;

const MESSAGE_TEMPLATE: &str = r#"<entry>
<title>{title}</title>
<link href="tbd"/>
<id>{entry_id}</id>
<updated>{updated_at}</updated>
<author>
<name>{author_name}</name>
<email>{author_email}</email>
</author>
<content type="text/plain">
{content}
</content>
</entry>
"#;

impl List {
    pub fn to_xml(&self) -> String {
        let mut entry_list = String::new();
        for msg in &self.recent_messages {
            entry_list.push_str(
                &template(
                    MESSAGE_TEMPLATE,
                    &[
                        ("title", &x(&msg.subject)),
                        // ("item_link", "sdf"),
                        ("entry_id", &x(&msg.id)),
                        ("updated_at", &Date::from(msg.received).rfc3339()),
                        (
                            "author_name",
                            &x(&msg.from.clone().name.unwrap_or(msg.from.clone().address)),
                        ),
                        ("author_email", &x(&msg.from.address)),
                        ("content", &x(&unformat_flowed(&msg.body))),
                    ],
                )
                .unwrap(),
            );
        }
        template(
            FEED_TEMPLATE,
            &[
                ("feed_link", "asdf"),
                ("feed_id", "asdf"),
                ("feed_title", "asdf"),
                ("last_updated", "adf"),
                ("entry_list", &entry_list),
                ("author_name", ""),
                ("author_email", ""),
            ],
        )
        .unwrap()
        // last_updated = time::secs_to_date(last_updated).rfc3339(),
    }
}

impl Thread {
    pub fn to_xml(&self) -> String {
        String::new()
    }
}
