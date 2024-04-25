// Simple threading algorithm based on https://datatracker.ietf.org/doc/html/rfc8621
// Only threads based on subject
// An alternative is implementing https://www.jwz.org/doc/threading.html which is a PITA
// A thread is a collection of messages sorted by date.
// Assumes msg can be found on disk at `path` -- should be made more abstract to handle other mail
// stores

use mail_parser::parsers::fields::thread::thread_name;
use mail_parser::Message;
use std::collections::HashMap;
use std::path::PathBuf;

pub type MessageId = String;

pub struct Msg {
    pub id: MessageId,
    pub path: PathBuf,
    pub time: mail_parser::DateTime,
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
        let msg_id = match msg.message_id() {
            Some(m) => m,
            None => return,
        };
        let time = match msg
            .received()
            .date()
            .or_else(|| msg.date())
        {
            Some(t) => t.clone(),
            None => return,
        };
        if self.id_index.get(msg_id).is_some() {
            // TODO handle duplicate msg case. Don't allow overwrites
            return;
        }
        let thread_name = thread_name(msg.subject().unwrap_or("(No Subject)"));

        let msg = Msg {
            id: msg_id.to_owned(),
            path,
            time,
        };
        let idx = self.subject_index.get(thread_name);

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
        for t in &mut self.threads {
            t.sort_by(|a, b| a.time.cmp(&b.time));
        }
    }
}
