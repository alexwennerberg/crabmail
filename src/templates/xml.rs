use super::util::xml_safe as x;
use crate::models::*;
use crate::time::Date;
use crate::util::unformat_flowed;
// use crate::templates::util::xml_safe;
// use anyhow::{Context, Result};

fn feed(
    feed_title: &str,
    feed_link: &str,
    updated: &str,
    author_name: &str,
    author_email: &str,
    entry_list: &str,
) -> String {
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
        <feed xmlns="http://www.w3.org/2005/Atom">
        <title>{feed_title}</title>
        <link href="{feed_link}"/>
        <updated>{updated}</updated>
        <author>
        <name>{author_name}</name>
        <email>{author_email}</email>
        </author>
        <id>{feed_link}</id>
        {entry_list}
        </feed>"#,
    )
}

fn message(
    title: &str,
    item_link: &str,
    entry_id: &str,
    updated: &str,
    author_name: &str,
    author_email: &str,
    content: &str,
) -> String {
    format!(
        r#"<entry>
        <title>{title}</title>
        <link href="{item_link}"/>
        <id>{entry_id}</id>
        <updated>{updated}</updated>
        <author>
        <name>{author_name}</name>
        <email>{author_email}</email>
        </author>
        <content type="text/plain">
        {content}
        </content>
        </entry>
        "#,
    )
}

impl StrMessage {
    pub fn to_xml(&self) -> String {
        let msg = self;
        let body = match self.flowed {
            true => unformat_flowed(&self.body),
            false => self.body.clone(),
        };
        message(
            &x(&msg.subject),
            &x(&self.url),
            &x(&msg.id),
            &Date::from(msg.received).rfc3339(),
            &x(&msg.from.clone().name.unwrap_or(msg.from.clone().address)),
            &x(&msg.from.address),
            &x(&body),
        )
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
        let last_updated = self.recent_messages.get(0).map(|x| x.received).unwrap_or(1);
        feed(
            &self.config.name,
            &self.url,
            &Date::from(last_updated).rfc3339(),
            &self.config.email,
            &self.config.email,
            &entry_list,
        )
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
        feed(
            &root.subject,
            &self.url,
            &Date::from(root.received).rfc3339(),
            root.from.name.as_ref().unwrap_or(&root.from.address),
            &root.from.address,
            &entry_list,
        )
    }
}
