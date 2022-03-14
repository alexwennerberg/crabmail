use crate::config::{Config, Subsection};
use crate::threading::{Msg, ThreadIdx};
use mail_parser::{Addr, HeaderValue, Message};
use std::borrow::Cow;
use std::path::PathBuf;

// messages are path-cleaned in this context (/ replaced)
// list_path = "/{list_name}/index.html"
// xml = "/{list_name}/atom.xml"
// thread_path = "/{list_name}/{thread_id}.html
// thread_xml = "/{list_name}/{thread_id}.xml
// raw_email = "/{list_name}/messages/{message_id}.eml
// paginate index somehow (TBD)

pub struct Lists {
    pub lists: Vec<List>,
    pub out_dir: PathBuf,
}

impl Lists {
    pub fn add(&mut self, thread_idx: ThreadIdx, name: &str) {
        // TODO safe name?
        let config = match Config::global().get_subsection(name) {
            Some(sub) => sub,
            None => Config::global().default_subsection(name),
        };
        self.lists.push(List {
            thread_idx,
            config,
            out_dir: self.out_dir.join(name),
        })
    }
}
pub struct List {
    pub thread_idx: ThreadIdx,
    // Thread topics
    pub config: Subsection, // path
    pub out_dir: PathBuf,
}

impl List {
    pub fn new(name: &str) -> Self {
        let con = Config::global();
        let sub: Subsection = match con.get_subsection(name) {
            Some(c) => c,
            None => con.default_subsection(name),
        };
        Self {
            thread_idx: ThreadIdx::default(),
            config: sub,
            out_dir: Config::global().out_dir.join(name),
        }
    }
}

pub struct Thread {
    pub messages: Vec<StrMessage>,
}

impl Thread {
    pub fn new(thread_idx: &Vec<Msg>) -> Self {
        let mut out = vec![];
        for m in thread_idx {
            let data = std::fs::read(&m.path).unwrap();
            let msg = StrMessage::new(&Message::parse(&data).unwrap());
            out.push(msg);
        }
        Thread { messages: out }
    }
}

// simplified, stringified-email for templating
// making everything owned because I'm l a z y
pub struct StrMessage {
    pub id: String,
    pub subject: String,
    pub from: MailAddress,
    pub date: String, // TODO better dates
    pub body: String,
    pub in_reply_to: Option<String>,
    // url: Cow<'a, str>,
    // download_path: PathBuf, // TODO
}

impl StrMessage {
    pub fn pathescape_msg_id(&self) -> PathBuf {
        PathBuf::from(self.id.replace("/", ";"))
    }
}

// i suck at Cow and strings
pub struct MailAddress {
    name: String,
    address: String,
}
impl MailAddress {
    fn from_addr(addr: &Addr) -> Self {
        // todo wtf
        let address = addr
            .address
            .clone()
            .unwrap_or(Cow::Borrowed("invalid-email"))
            .to_string();
        MailAddress {
            name: addr
                .name
                .clone()
                .unwrap_or(Cow::Owned(address.clone()))
                .to_string(),
            address: address.to_string(),
        }
    }
}

// TODO rename
impl StrMessage {
    pub fn new(msg: &Message) -> StrMessage {
        let id = msg.get_message_id().unwrap_or("");
        let subject = msg.get_subject().unwrap_or("(No Subject)");
        let invalid_email = Addr::new(None, "invalid-email");
        let from = match msg.get_from() {
            HeaderValue::Address(fr) => fr,
            _ => &invalid_email,
        };
        let from = MailAddress::from_addr(from);
        let date = msg.get_date().unwrap().to_iso8601();
        let in_reply_to = msg
            .get_in_reply_to()
            .as_text_ref()
            .and_then(|a| Some(a.to_string()));

        // TODO linkify body
        // TODO unformat-flowed
        let body = msg
            .get_text_body(0)
            .unwrap_or(Cow::Borrowed("[No message body]"));
        StrMessage {
            id: id.to_owned(),
            subject: subject.to_owned(),
            from: from,
            date: date.to_owned(),
            body: body.to_string(),
            in_reply_to: in_reply_to,
        }
    }
}
