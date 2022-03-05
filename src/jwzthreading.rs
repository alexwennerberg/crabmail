// jwz threading https://www.jwz.org/doc/threading.html
//
//
// implementing this in Rust is a nightmare and makes me feel bad about myself so I am probably
// going to do something simpler

use anyhow::{Context, Result};
use mail_parser::parsers::fields::thread::thread_name;
use mail_parser::Message;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Display;
use std::rc::{Rc, Weak};

#[derive(Default, Clone)]
struct JwzContainer {
    message: Option<JwzMessage>,
    parent: Option<MessageId>,
    children: Vec<MessageId>,
    next: Option<MessageId>,
}

impl JwzContainer {}

#[derive(Default, Clone)]
pub struct JwzMessage {
    id: String,
    subject: String,
    references: Vec<String>,
}

impl JwzMessage {
    // TODO move out of here
    pub fn parse(msg: Message) -> Result<Self> {
        let id = msg
            .get_message_id()
            .context("Missing message ID")?
            .to_owned();
        let subject = msg.get_subject().context("Missing subject")?.to_owned();
        let references = vec![];
        Ok(JwzMessage {
            id,
            subject,
            references,
        })
    }
}

type MessageId = String;

#[derive(Default, Clone)]
pub struct List {
    id_table: HashMap<MessageId, JwzContainer>,
    subject_table: HashMap<String, MessageId>,
}

impl List {
    pub fn new() -> Self {
        List::default()
    }

    // Todo enumerate errors or something
    pub fn add_email(&mut self, jwz_msg: JwzMessage) {
        let msg_id = jwz_msg.id.clone();
        let references = jwz_msg.references.clone();
        // 1A
        if self
            .id_table
            .get(&msg_id)
            .and_then(|c| Some(c.message.is_none()))
            == Some(true)
        {
            let cont = self.id_table.get_mut(&msg_id).unwrap();
            cont.message = Some(jwz_msg)
        } else {
            let new_container = JwzContainer {
                message: Some(jwz_msg),
                ..Default::default()
            };
            self.id_table.insert(msg_id.clone(), new_container);
        }
        // 1B
        for pair in references.windows(2) {
            // TODO check loop
            let parent = self.container_from_id(&pair[0]);
            if !parent.children.contains(&pair[1]) {
                parent.children.push(pair[1].to_owned());
            }
            let child = self.container_from_id(&pair[1]);
            child.parent = Some(pair[0].to_owned());
        }

        // 1C
        if references.len() > 0 {
            let container = self.container_from_id(&msg_id);
            container.parent = Some(references[references.len() - 1].clone());
        }

        // 2-4
        let root: Vec<&JwzContainer> = self
            .id_table
            .iter()
            .filter_map(|(k, v)| {
                if v.parent.is_none() {
                    return Some(v);
                }
                return None;
            })
            .filter(|c| c.children.len() > 0)
            // TODO Filter and promote if no message (4B)
            .collect();

        // 5
        for item in root {
            // TODO If there is no message in the Container...
            // ^^^ WHY WOULD THIS HAPPEN JWZ??
            if let Some(i) = &item.message {
                let threadn = thread_name(&i.subject);
                if threadn == "" {
                    continue;
                }
            }
        }
    }

    fn container_from_id(&mut self, msg_id: &str) -> &mut JwzContainer {
        match self.id_table.get(msg_id) {
            Some(c) => self.id_table.get_mut(msg_id).unwrap(),
            None => {
                self.id_table
                    .insert(msg_id.to_string(), JwzContainer::default());
                self.id_table.get_mut(msg_id).unwrap()
            }
        }
    }

    pub fn finalize(&mut self) {}
}
