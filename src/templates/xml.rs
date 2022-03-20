// use super::util::xml_escape;
use crate::models::*;
// use crate::templates::util::xml_safe;
// use anyhow::{Context, Result};
// use nanotemplate::template;

// const ATOM_ENTRY_LIMIT: i32 = 100;

// impl List {
//     fn to_xml(&self) {
//         template(
//             r#"<?xml version="1.0" encoding="utf-8"?>
// <feed xmlns="http://www.w3.org/2005/Atom">
// <title>{feed_title}</title>
// <link href="{feed_link}"/>
// <updated>{last_updated}</updated>
// <author>
// <name>{author_name}</name>
// <email>{author_email}</email>
// </author>
// <id>{feed_id}</id>
// {entry_list}
// </feed>"#,
//             &[],
//             // feed_title = &self.name,
//             // feed_link = &self.url,
//             // last_updated = time::secs_to_date(last_updated).rfc3339(),
//             // author_name = &self.email,
//             // author_email = &self.email,
//             // feed_id = &self.url,
//             // entry_list = entries_str,
//         )
//     }
// }

impl Thread {
    pub fn to_xml(&self) -> String {
        String::new()
    }
}
//         return ""
//         for message in &self.messages {
//             let tmpl = self.build_msg_atom(message);
//             entries.push_str(&tmpl);
//         }
//         let root = self.messages[0];
//         let atom = format!(
//             r#"<?xml version="1.0" encoding="utf-8"?>
// <feed xmlns="http://www.w3.org/2005/Atom">
// <title>{feed_title}</title>
// <link rel="self" href="{feed_link}"/>
// <updated>{last_updated}</updated>
// <author>
// <name>{author_name}</name>
// <email>{author_email}</email>
// </author>
// <id>{feed_id}</id>
// {entry_list}
// </feed>"#,
//             feed_title = xml_safe(&root.subject),
//             feed_link = self.url(),
//             last_updated = time::secs_to_date(self.last_reply()).rfc3339(),
//             author_name = xml_safe(short_name(&root.from)),
//             author_email = xml_safe(&root.from.addr),
//             feed_id = self.url(),
//             entry_list = entries,
//         );
//     }
// }

// impl<'a> StrMessage<'a> {
//     fn to_xml(&self) -> String {
//         template(
//             r#"<entry>
// <title>{title}</title>
// <link href="{item_link}"/>
// <id>{entry_id}</id>
// <updated>{updated_at}</updated>
// <author>
// <name>{author_name}</name>
// <email>{author_email}</email>
// </author>
// <content type="text/plain">
// {content}
// </content>
// </entry>
// "#,
//             &[
//                 ("title", self.subject.as_ref()),
//                 // ("item_link", "TBD"), -> this introduces filesystem dependency
//                 // ("entry_id", self.id),
//                 ("updated_at", "TBD"),
//                 // ("author_name", self.from.name),
//                 // ("author_email", self.from.address),
//                 // ("content", self.body),
//             ],
//         )
//         .unwrap()
//     }
// }
