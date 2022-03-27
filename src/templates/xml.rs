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
<link href="{item_link}"/>
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

impl StrMessage {
    pub fn to_xml(&self) -> String {
        let msg = self;
        let body = match self.flowed {
            true => unformat_flowed(&self.body),
            false => self.body.clone(),
        };
        template(
            MESSAGE_TEMPLATE,
            &[
                ("title", &x(&msg.subject)),
                ("item_link", &x(&self.url)),
                ("entry_id", &x(&msg.id)),
                ("updated_at", &Date::from(msg.received).rfc3339()),
                (
                    "author_name",
                    &x(&msg.from.clone().name.unwrap_or(msg.from.clone().address)),
                ),
                ("author_email", &x(&msg.from.address)),
                ("content", &x(&body)),
            ],
        )
        .unwrap()
    }
}

// TODO dedup
impl List {
    pub fn to_xml(&self) -> String {
        let mut entry_list = String::new();
        for msg in &self.recent_messages {
            entry_list.push_str(&msg.to_xml());
        }
        // Sometimes its unclear whether to do stuff like this in models.rs or here. could refactor
        let last_updated = self
            .recent_messages
            .get(0)
            .and_then(|x| Some(x.received))
            .unwrap_or(1);
        template(
            FEED_TEMPLATE,
            &[
                ("feed_link", &self.url),
                ("feed_id", &self.url),
                ("feed_title", &self.config.name),
                ("last_updated", &Date::from(last_updated).rfc3339()),
                ("entry_list", &entry_list),
                ("author_name", &self.config.email),
                ("author_email", &self.config.email),
            ],
        )
        .unwrap()
    }
}

impl Thread {
    pub fn to_xml(&self) -> String {
        let mut entry_list = String::new();
        for msg in &self.messages {
            entry_list.push_str(&msg.to_xml());
        }
        // Sometimes its unclear whether to do stuff like this in models.rs or here. could refactor
        let root = &self.messages[0];
        template(
            FEED_TEMPLATE,
            &[
                ("feed_link", &self.url),
                ("feed_id", &self.url),
                ("feed_title", &root.subject),
                ("last_updated", &Date::from(root.received).rfc3339()),
                ("entry_list", &entry_list),
                (
                    "author_name",
                    &root.from.name.clone().unwrap_or(root.from.address.clone()),
                ),
                ("author_email", &root.from.address),
            ],
        )
        .unwrap()
    }
}
