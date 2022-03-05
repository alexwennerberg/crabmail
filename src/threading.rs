// Simple threading algorithm based on https://datatracker.ietf.org/doc/html/rfc8621
// A thread is a collection of messages sorted by date.
// Assumes msg can be found on disk at `path` -- could be made more abstract

use mail_parser::parsers::fields::thread::thread_name;
use mail_parser::{DateTime, Message};
use std::collections::HashMap;
use std::path::PathBuf;

pub type MessageId = String;

pub struct Msg {
    pub id: MessageId,
    pub path: PathBuf,
}

impl Msg {}

#[derive(Default)]
pub struct ThreadIdx {
    pub threads: Vec<Vec<Msg>>,
    id_index: HashMap<MessageId, usize>,
    subject_index: HashMap<String, usize>,
}

impl ThreadIdx {
    pub fn new() -> Self {
        ThreadIdx::default()
    }

    // Todo enumerate errors or something
    // TODO should be format agnostic (use internal representation of email)
    pub fn add_email(&mut self, msg: &Message, path: PathBuf) {
        let msg_id = msg.get_message_id().unwrap(); // TODO unwrap
                                                    // TODO handle duplicate id case
        let received = msg
            .get_received()
            .as_datetime_ref()
            .or_else(|| msg.get_date())
            .unwrap(); // TODO fix unwrap
        let in_reply_to = msg.get_in_reply_to().as_text_ref();
        let last_reference = msg.get_in_reply_to().as_text_ref();
        let thread_name = thread_name(msg.get_subject().unwrap_or("(No Subject)"));

        let msg = Msg {
            id: msg_id.to_owned(),
            path,
        };
        let reference = in_reply_to.or_else(|| last_reference);

        let idx = match reference {
            Some(id) => self.id_index.get(id),
            None => self.subject_index.get(thread_name),
        };
        let id = match idx {
            Some(i) => {
                self.threads[*i].push(msg);
                *i
            }
            None => {
                self.threads.push(vec![msg]);
                self.threads.len() - 1
            }
        };
        self.id_index.insert(msg_id.to_string(), id);
        self.subject_index.insert(thread_name.to_string(), id);
    }

    pub fn finalize(&mut self) {
        // TODO sort thread list by last reply date
    }
}
