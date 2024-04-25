use crate::config::{Config, Subsection};
use crate::threading::{Msg, ThreadIdx};
use mail_builder::headers::date::Date;
use mail_builder::MessageBuilder;
use mail_parser::{Addr, DateTime, HeaderValue, MimeHeaders, Message, RfcHeader};
use std::borrow::Cow;
use std::path::PathBuf;

// messages are path-cleaned in this context (/ replaced)
// list_path = "/{list_name}/index.html"
// xml = "/{list_name}/atom.xml"
// thread_path = "/{list_name}/{thread_id}.html
// thread_xml = "/{list_name}/{thread_id}.xml
// raw_email = "/{list_name}/messages/{message_id}.mbox

// TODO a better way to handle these is to use lifetimes rather than ownership
// I should implement an iterator that writes each message without holding them in memory probably
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
        let url = format!("{}/{}", Config::global().base_url, config.name);
        self.lists.push(List {
            thread_idx,
            config,
            url,
            thread_topics: vec![],
            recent_messages: vec![],
            out_dir: self.out_dir.join(name),
        })
    }
}
pub struct List {
    pub thread_idx: ThreadIdx,
    pub thread_topics: Vec<ThreadSummary>, // TODO
    pub recent_messages: Vec<StrMessage>,
    pub config: Subsection, // path
    pub out_dir: PathBuf,
    pub url: String,
}

// doesnt include full msg data
pub struct ThreadSummary {
    pub message: StrMessage,
    pub reply_count: u64,
    pub last_reply: DateTime,
}

pub struct Thread {
    pub messages: Vec<StrMessage>,
    pub url: String,
}

impl Thread {
    pub fn new(thread_idx: &Vec<Msg>, list_name: &str, list_email: &str) -> Self {
        let mut messages = vec![];
        for m in thread_idx {
            let data = std::fs::read(&m.path).unwrap();
            let mut msg = StrMessage::new(&Message::parse(&data).unwrap());
            msg.mailto = msg.mailto(list_name, list_email);
            messages.push(msg);
        }
        let url = format!(
            "{}/{}/{}",
            Config::global().base_url,
            list_name,
            messages[0].pathescape_msg_id().to_str().unwrap(),
        );
        Thread { url, messages }
    }
}

// simplified, stringified-email for templating
// making everything owned because I'm l a z y
#[derive(Debug, Clone)]
pub struct StrMessage {
    pub id: String,
    pub subject: String,
    pub thread_subject: String,
    pub received: DateTime,
    pub preview: String,
    pub from: MailAddress,
    pub date: DateTime,
    pub body: String,
    pub flowed: bool,
    pub mailto: String, // mailto link
    pub in_reply_to: Option<String>,
    pub to: Vec<MailAddress>,
    pub cc: Vec<MailAddress>,
    pub url: String,
}

// i suck at Cow and strings
#[derive(Debug, Clone)]
pub struct MailAddress {
    pub name: Option<String>,
    pub address: String,
}
impl MailAddress {
    fn from_addr(addr: &Addr) -> Self {
        MailAddress {
            name: addr.name.as_ref().map(|a| a.to_string()),
            address: addr.address.as_ref().unwrap().to_string(),
        }
    }
}

impl std::fmt::Display for MailAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(n) = &self.name {
            write!(f, "\"{}\" ", n)?;
        }
        write!(f, "<{}>", self.address)
    }
}

// TODO rename
impl StrMessage {
    pub fn pathescape_msg_id(&self) -> PathBuf {
        // use at your own risk on windows. idk how safe filepaths look there.
        PathBuf::from(self.id.replace('/', ";"))
    }
    // wonky
    // for some reason mbox is used over eml for things like git, mutt, etc
    pub fn export_mbox(&self) -> Vec<u8> {
        let mut message = MessageBuilder::new();
        if self.flowed {
            message.format_flowed();
        }
        let from = self.from.name.clone().unwrap_or_default();
        message.message_id(self.id.as_str());
        message.from((from.as_str(), self.from.address.as_str()));
        // TODO no alloc. No copy pasta

        fn map_addrs(addrs: &[MailAddress]) -> mail_builder::headers::address::Address<'_> {
            addrs
                .iter()
                .map(|addr| {
                    (
                        addr.name.as_ref().map_or("", |s| s.as_str()),
                        addr.address.as_str(),
                    )
                })
                .collect::<Vec<_>>()
                .into()
        }

        message.to(map_addrs(&self.to));
        message.cc(map_addrs(&self.cc));

        message.header("Date", Date::new(self.date.to_timestamp()));
        if let Some(irt) = &self.in_reply_to {
            message.in_reply_to(irt.as_str());
        }
        // list-archive
        message.subject(&self.subject);
        // Figure out body export and content-transfer...
        message.text_body(&self.body);
        // Dummy data for mbox
        let mut output = Vec::from(&b"From mboxrd@z Thu Jan  1 00:00:00 1970\n"[..]);
        message.write_to(&mut output).unwrap();
        // for mbox
        output.push(b'\n');

        output
    }

    pub fn mailto(&self, list_name: &str, list_email: &str) -> String {
        let mut url = format!("mailto:{}?", list_email);

        let from = self.from.address.clone();
        // make sure k is already urlencoded
        let mut pushencode = |k: &str, v| {
            url.push_str(&format!("{}={}&", k, urlencoding::encode(v)));
        };
        let fixed_id = format!("<{}>", &self.id);
        pushencode("cc", &from);
        pushencode("in-reply-to", &fixed_id);
        let list_url = format!("{}/{}", &Config::global().base_url, list_name);
        pushencode("list-archive", &list_url);
        pushencode("subject", &format!("Re: {}", self.thread_subject));
        // quoted body
        url.push_str("body=");
        for line in self.body.lines() {
            url.push_str("%3E%20");
            url.push_str(&urlencoding::encode(line));
            url.push_str("%0A");
        }
        url
    }

    // only place that depends on list and thread. hmm
    pub fn set_url(&mut self, list: &List, thread: &ThreadSummary) {
        self.url = format!(
            "{}/{}/{}#{}",
            Config::global().base_url,
            list.config.name,
            thread.message.pathescape_msg_id().to_str().unwrap(),
            self.id
        );
    }

    pub fn new(msg: &Message) -> StrMessage {
        let id = msg.message_id().unwrap_or("");
        // TODO duplicate in threading.rs
        let received = msg
            .received()
            .as_datetime_ref()
            .or_else(|| msg.date())
            .unwrap()
            .clone();
        let subject = msg.subject().unwrap_or("(No Subject)");
        let thread_subject = msg.thread_name().unwrap_or("(No Subject)");
        let invalid_email = Addr::new(None, "invalid-email");
        let preview = match msg.body_preview(80) {
            Some(b) => b.to_string(),
            None => String::new(),
        };
        let from = match msg.from() {
            HeaderValue::Address(fr) => fr,
            _ => &invalid_email,
        };
        let from = MailAddress::from_addr(from);
        let date = msg.date().cloned().unwrap_or(crate::util::EPOCH);

        /// Turns a header value into a list of addresses
        fn addr_list(header: &HeaderValue) -> Vec<MailAddress> {
            match header {
                HeaderValue::Address(addr) => vec![MailAddress::from_addr(addr)],
                HeaderValue::AddressList(addrs) => {
                    addrs.iter().map(MailAddress::from_addr).collect()
                }
                _ => vec![], // TODO: should this be `unreachable!`?
            }
        }

        let to = addr_list(msg.to());
        let cc = addr_list(msg.cc());

        let in_reply_to = msg.in_reply_to().as_text_ref().map(|a| a.to_string());

        // TODO linkify body
        // TODO unformat-flowed
        let body = msg
            .body_text(0)
            .unwrap_or(Cow::Borrowed("[No message body]"));

        // life is a nightmare
        let flowed = msg.content_type()
            .map_or(false, |x| x.c_type == "flowed");
        StrMessage {
            id: id.to_owned(),
            subject: subject.to_owned(),
            from,
            received,
            preview,
            to,
            cc,
            url: String::new(),
            thread_subject: thread_subject.to_owned(),
            date,
            body: body.to_string(),
            flowed,
            mailto: String::new(),
            in_reply_to,
        }
    }
}
