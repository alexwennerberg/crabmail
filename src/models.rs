use crate::config::{Config, Subsection};
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

pub struct Lists<'a> {
    pub lists: Vec<List<'a>>,
    pub out_dir: PathBuf,
}

pub struct List<'a> {
    pub threads: Vec<Thread<'a>>,
    pub config: Subsection, // path
    pub out_dir: PathBuf,
}

pub struct Thread<'a> {
    pub messages: Vec<StrMessage<'a>>,
}

impl Thread<'_> {
    // fn new() -> Self {
    // Thread {messagse: }
    // }
}

// TODO rename
// simplified, stringified-email for templating
pub struct StrMessage<'a> {
    pub id: Cow<'a, str>,
    pub subject: Cow<'a, str>,
    pub from: MailAddress,
    pub date: Cow<'a, str>, // TODO better dates
    pub body: Cow<'a, str>,
    pub in_reply_to: Option<Cow<'a, str>>,
    // url: Cow<'a, str>,
    // download_path: PathBuf, // TODO
}

impl StrMessage<'_> {
    // Raw file path
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
impl<'a> Thread<'a> {
    fn new_message(msg: &'a Message<'a>) -> StrMessage<'a> {
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
            .and_then(|a| Some(Cow::Borrowed(a)));

        // TODO linkify body
        // TODO unformat-flowed
        let body = msg
            .get_text_body(0)
            .unwrap_or(Cow::Borrowed("[No message body]"));
        StrMessage {
            id: Cow::Borrowed(id),
            subject: Cow::Borrowed(subject),
            from: from,
            date: Cow::Owned(date),
            body: body,
            in_reply_to: in_reply_to,
        }
    }
}
