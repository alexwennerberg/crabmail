use crate::config::{Config, Subsection};
use crate::threading::{Msg, ThreadIdx};
use mail_builder::headers::text::Text;
use mail_builder::MessageBuilder;
use mail_parser::{Addr, HeaderValue, Message, RfcHeader};
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
    pub last_reply: i64, // unix
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
    pub received: i64,
    pub preview: String,
    pub from: MailAddress,
    pub date: String, // TODO better dates
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
        // todo wtf
        let address = addr.address.to_owned();
        MailAddress {
            name: addr.name.to_owned().and_then(|a| Some(a.to_string())),
            address: address.unwrap().to_string(),
        }
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        if let Some(n) = &self.name {
            out.push('"');
            out.push_str(&n);
            out.push('"');
            out.push(' ');
        }
        out.push('<');
        out.push_str(&self.address);
        out.push('>');
        out
    }
}

// TODO rename
impl StrMessage {
    pub fn pathescape_msg_id(&self) -> PathBuf {
        PathBuf::from(self.id.replace("/", ";"))
    }
    // wonky
    // for some reason mbox is used over eml for things like git, mutt, etc
    pub fn export_mbox(&self) -> Vec<u8> {
        let mut message = MessageBuilder::new();
        if self.flowed {
            message.format_flowed();
        }
        let from = self.from.name.clone().unwrap_or(String::new());
        message.message_id(self.id.as_str());
        message.from((from.as_str(), self.from.address.as_str()));
        // TODO no alloc. No copy pasta
        message.to(self
            .to
            .iter()
            .map(|x| (x.name.clone().unwrap_or(String::new()), x.address.clone()))
            .collect::<Vec<(String, String)>>());
        message.cc(self
            .cc
            .iter()
            .map(|x| (x.name.clone().unwrap_or(String::new()), x.address.clone()))
            .collect::<Vec<(String, String)>>());
        message.header("Date", Text::from(self.date.as_str()));
        if let Some(irt) = &self.in_reply_to {
            message.in_reply_to(irt.as_str());
        }
        // list-archive
        message.subject(&self.subject);
        // Figure out body export and content-transfer...
        message.text_body(&self.body);
        let mut output = Vec::new();
        message.write_to(&mut output).unwrap();
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
            url.push_str(&urlencoding::encode(&line));
            url.push_str("%0A");
        }
        url.into()
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
        let id = msg.get_message_id().unwrap_or("");
        // TODO duplicate in threading.rs
        let received = msg
            .get_received()
            .as_datetime_ref()
            .or_else(|| msg.get_date())
            .unwrap()
            .to_timestamp()
            .unwrap_or(-1);
        let subject = msg.get_subject().unwrap_or("(No Subject)");
        let thread_subject = msg.get_thread_name().unwrap_or("(No Subject)");
        let invalid_email = Addr::new(None, "invalid-email");
        let preview = match msg.get_body_preview(80) {
            Some(b) => b.to_string(),
            None => String::new(),
        };
        let from = match msg.get_from() {
            HeaderValue::Address(fr) => fr,
            _ => &invalid_email,
        };
        let from = MailAddress::from_addr(from);
        let date = msg
            .get_rfc_header(RfcHeader::Date)
            .and_then(|x| Some(x.get(0).unwrap_or(&Cow::from("")).to_string()))
            .unwrap_or(String::new())
            .trim()
            .to_owned(); // TODO awkawrd
        let to = match msg.get_to() {
            HeaderValue::Address(fr) => vec![MailAddress::from_addr(fr)],
            HeaderValue::AddressList(fr) => fr.iter().map(|a| MailAddress::from_addr(a)).collect(),
            _ => vec![],
        };
        // todo no copypaste
        let cc = match msg.get_cc() {
            HeaderValue::Address(fr) => vec![MailAddress::from_addr(fr)],
            HeaderValue::AddressList(fr) => fr.iter().map(|a| MailAddress::from_addr(a)).collect(),
            _ => vec![],
        };
        let in_reply_to = msg
            .get_in_reply_to()
            .as_text_ref()
            .and_then(|a| Some(a.to_string()));

        // TODO linkify body
        // TODO unformat-flowed
        let body = msg
            .get_text_body(0)
            .unwrap_or(Cow::Borrowed("[No message body]"));

        // life is a nightmare
        let flowed = msg
            .get_text_part(0)
            .and_then(|x| x.headers_rfc.get(&RfcHeader::ContentType))
            .and_then(|x| x.as_content_type_ref())
            .and_then(|x| x.attributes.as_ref())
            .and_then(|x| x.get("format"))
            .and_then(|x| Some(x == "flowed"))
            .unwrap_or(false);
        StrMessage {
            id: id.to_owned(),
            subject: subject.to_owned(),
            from: from,
            received,
            preview,
            to,
            cc,
            url: String::new(),
            thread_subject: thread_subject.to_owned(),
            date: date.to_owned(),
            body: body.to_string(),
            flowed,
            mailto: String::new(),
            in_reply_to: in_reply_to,
        }
    }
}
