use anyhow::{anyhow, Context, Result};
use horrorshow::helper::doctype;
use horrorshow::owned_html;
use horrorshow::prelude::*;
use horrorshow::Template;
use std::io;
use std::io::BufWriter;
use std::path::Path;

#[macro_use]
extern crate horrorshow;

use mailparse::*;
use mbox_reader::MboxFile;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use urlencoding;

use config::{Config, INSTANCE};
mod config;
mod utils;

// TODO be more clear about the expected input types
// maildi

// Not a "raw email" struct, but an email object that can be represented by
// crabmail.
#[derive(Debug, Clone)]
struct Email {
    // TODO allocs
    id: String,
    from: SingleInfo,
    subject: String,
    in_reply_to: Option<String>,
    date: u64, // unix epoch. received date
    body: String,
    mime: String,
}

#[derive(Debug, Clone)]
struct MailThread<'a> {
    messages: Vec<&'a Email>, // sorted
    hash: String,
    last_reply: u64,
}

fn layout(page_title: impl Render, content: impl Render) -> impl Render {
    // owned_html _moves_ the arguments into the template. Useful for returning
    // owned (movable) templates.
    owned_html! {
            : doctype::HTML;
            html {
            head {
                title : &page_title;
                : Raw("<meta http-equiv='Permissions-Policy' content='interest-cohort=()'/>
                        <link rel='stylesheet' type='text/css' href='style.css' />
                        <meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0,user-scalable=0' />
                        <link rel='icon' href='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📧</text></svg>'>");
                meta(name="description", content=&page_title);
            }
            body {
                main {
                :&content
                }
                hr;
            div(class="footer") {
    : Raw("Archive generated with  <a href='https://git.alexwennerberg.com/crabmail/'>crabmail</a>")
            }
            }
            }
        }
}

struct ThreadList<'a> {
    threads: Vec<MailThread<'a>>,
}

// Get short name from an address name like "alex wennerberg <alex@asdfasdfafd>"
fn short_name(s: &SingleInfo) -> &str {
    match &s.display_name {
        Some(dn) => dn,
        None => &s.addr
    }
}
impl<'a> ThreadList<'a> {
    pub fn write_to_file(&self, out_dir: &Path) -> Result<()> {
        let tmp = html! {
            h1(class="page-title"): &Config::global().list_name;
            a(href=format!("mailto:{}", &Config::global().list_email)) {
                : &Config::global().list_email
            }
            hr;
            @ for thread in &self.threads {
                div(class="message-sum") {
                    a(class="threadlink", href=format!("threads/{}.html", urlencoding::encode(&thread.messages[0].id))) {
                        : &thread.messages[0].subject
                    }
                    br;
                    a(class="addr", href=format!("mailto:/{}", &thread.messages[0].from.addr)){
                        : short_name(&thread.messages[0].from)
                    }

                span(class="timeago") {
                    : format!(" | {} replies | {}", thread.messages.len() - 1, utils::timeago(thread.last_reply()))
                }
                }
            }
        };

        let mut file = File::create(&out_dir.join("index.html"))?;
        let mut br = BufWriter::new(file);
        layout(Config::global().list_name.as_str(), tmp).write_to_io(&mut br)?;
        Ok(())
    }
}

impl<'a> MailThread<'a> {
    pub fn last_reply(&self) -> u64 {
        return self.messages[self.messages.len() - 1].date;
    }

    fn build_atom_feed() -> String {
        String::new()
    }

    fn write_to_file(&self, out_dir: &Path) -> Result<()> {
        let root = self.messages[0];
        let tmp = html! {
            h1(class="page-title"): &root.subject;
            div {
                @ for message in &self.messages {
                    hr;
                    div(id=&message.id, class="message") {
                        a(href=format!("mailto:{}", &message.from.addr), class="addr") {
                            : &message.from.to_string();
                        }
                    }
                    span(class="timeago") {
                        : utils::timeago(message.date)
                    }
                    a(title="permalink", href=format!("#{}", &message.id)) {
                        : "🔗" 
                    }
                    @ if message.in_reply_to.is_some() { // TODO figure out match
                        a(title="replies-to", href=format!("#{}", message.in_reply_to.clone().unwrap())){
                            : "Re:"
                        }
                    }
                    div(class="email-body") {
                        : Raw(utils::email_body(&message.body))
                    }
                    div(class="right"){
                        a (href=message.mailto()) {
                            :"✉️ reply"
                        }
                    }
                }
            }
        };
        let thread_dir = out_dir.join("threads");
        std::fs::create_dir(&thread_dir).ok();

        let mut file = File::create(&thread_dir.join(format!("{}.html", &self.hash)))?;
        let mut br = BufWriter::new(file);
        layout(root.subject.as_str(), tmp).write_to_io(&mut br)?;
        Ok(())
    }
}

impl Email {
    // mailto:... populated with everything you need
    pub fn mailto(&self) -> String {
        // TODO configurable
        let mut url = format!("mailto:{}?", Config::global().list_email);

        let from = self.from.to_string();
        // make sure k is already urlencoded
        let mut pushencode = |k: &str, v| {
            url.push_str(&format!("{}={}&", k, urlencoding::encode(v)));
        };
        pushencode("cc", &from);
        pushencode("in-reply-to", &self.id);
        pushencode("subject", &format!("Re: {}", self.subject));
        url.pop();
        url.into()
    }

    // Build hash string from message ID
    // This allows for a stable, url-friendly filename
    pub fn hash(&self) -> String {
        let mut hasher = Shake128::default();
        hasher.update(&self.id.as_bytes());
        let mut reader = hasher.finalize_xof();
        let mut res1 = [0u8; 6];
        XofReader::read(&mut reader, &mut res1);
        let mut out = String::new();
        for byte in &res1 {
            use std::fmt::Write; // TODO
            write!(out, "{:02x}", byte).unwrap();
        }
        return out;
    }
}

#[cfg(feature = "html")]
fn parse_html_body(email: &ParsedMail) -> String {
    use std::collections::HashSet;
    use std::iter::FromIterator;
    // TODO dont initialize each time
    // TODO sanitize id, classes, etc.
    let tags = HashSet::from_iter(vec!["a", "b", "i", "br", "p", "span", "u"]);
    let a = ammonia::Builder::new()
        .tags(tags)
        .clean(&email.get_body().unwrap_or("".to_string()))
        .to_string();
    a
}

fn local_parse_email(data: &[u8]) -> Result<Email> {
    let parsed_mail = parse_mail(data)?;
    let mut body: String = "[Message has no body]".to_owned();
    let mut mime: String = "".to_owned();
    let nobody = "[No body found]";
    if parsed_mail.subparts.len() == 0 {
        body = parsed_mail.get_body().unwrap_or(nobody.to_owned());
    } else {
        for sub in &parsed_mail.subparts {
            if sub.ctype.mimetype == "text/plain" {
                body = sub.get_body().unwrap_or(nobody.to_owned());
                mime = sub.ctype.mimetype.clone();
                break;
            }
        }
        #[cfg(feature = "html")]
        for sub in &parsed_mail.subparts {
            if sub.ctype.mimetype == "text/html" {
                mime = sub.ctype.mimetype.clone();
                break;
            }
        }
    }
    let headers = parsed_mail.headers;
    let id = headers
        .get_first_value("message-id")
        .context("No message ID")?;
    if id.contains("..") {
        // dont hack me
        // id goes into filename. TODO more verification
        return Err(anyhow!("bad message ID"));
    }
    // Assume 1 in-reply-to header. a reasonable assumption
    let in_reply_to = headers.get_first_value("in-reply-to");
    let subject = headers
        .get_first_value("subject")
        .unwrap_or("(no subject)".to_owned());
    // TODO not guaranteed to be accurate. Maybe use "received"?
    let date_string = &headers.get_first_value("date").context("No date header")?;
    let date = dateparse(date_string)? as u64;
    let from = addrparse_header(headers.get_first_header("from").context("No from header")?)?
        .extract_single_info()
        .context("Could not parse from header")?;

    return Ok(Email {
        id,
        in_reply_to,
        from,
        subject,
        date,
        body,
        mime,
    });
}

const HELP: &str = "\
Usage: crabmail 

-m --mbox   input mbox file
-c --config config file [crabmail.conf]
-d --dir    output directory [site]
";

fn main() -> Result<()> {
    let mut pargs = pico_args::Arguments::from_env();

    if pargs.contains(["-h", "--help"]) {
        print!("{}", HELP);
        std::process::exit(0);
    }
    // TODO configurable
    let out_dir = pargs
        .opt_value_from_os_str(["-d", "--dir"], parse_path)?
        .unwrap_or("site".into());
    let config_file = pargs
        .opt_value_from_os_str(["-c", "--config"], parse_path)?
        .unwrap_or("crabmail.conf".into());
    let in_mbox = pargs.value_from_os_str(["-m", "--mbox"], parse_path)?;

    let config = Config::from_file(&config_file).unwrap(); // TODO better err handling
    INSTANCE.set(config).unwrap();

    let mbox = MboxFile::from_file(&in_mbox)?;

    let mut thread_index: HashMap<String, Vec<String>> = HashMap::new();

    let mut email_index: HashMap<String, Email> = HashMap::new();
    for entry in mbox.iter() {
        let buffer = entry.message().unwrap();
        let email = match local_parse_email(buffer) {
            Ok(e) => e,
            Err(e) => {
                println!("{:?}", e);
                continue;
            }
        };
        // TODO fix borrow checker
        if let Some(reply) = email.in_reply_to.clone() {
            match thread_index.get(&reply) {
                Some(_) => {
                    let d = thread_index.get_mut(&reply).unwrap();
                    d.push(email.id.clone());
                }
                None => {
                    thread_index.insert(reply, vec![email.id.clone()]);
                }
            }
        }
        email_index.insert(email.id.clone(), email);
    }

    let mut thread_roots: Vec<Email> = email_index
        .iter()
        .filter_map(|(_, v)| {
            if v.in_reply_to.is_none() {
                return Some(v.clone());
            }
            return None;
        })
        .collect();
    std::fs::create_dir(&out_dir).ok();
    let mut threads = vec![];
    for root in &mut thread_roots {
        let mut thread_ids = vec![];
        let mut current: Vec<String> = vec![root.id.clone()];
        while current.len() > 0 {
            let top = current.pop().unwrap().clone();
            thread_ids.push(top.clone());
            if let Some(ids) = thread_index.get(&top.clone()) {
                for item in ids {
                    current.push(item.to_string());
                }
            }
        }

        let mut messages: Vec<&Email> = thread_ids
            .iter()
            .map(|id| email_index.get(id).unwrap())
            .collect();

        messages.sort_by_key(|a| a.date);

        let mut thread = MailThread {
            messages: messages,
            hash: root.hash(),
            last_reply: 0, // TODO
        };

        thread.last_reply = thread.last_reply();

        thread.write_to_file(&out_dir);
        threads.push(thread);
    }

    threads.sort_by_key(|a| a.last_reply);
    threads.reverse();
    ThreadList{threads}.write_to_file(&out_dir);
    // kinda clunky
    let css = include_bytes!("style.css");
    let mut css_root = File::create(out_dir.join("style.css"))?;
    css_root.write(css);
    let mut css_sub = File::create(out_dir.join("threads").join("style.css"))?;
    css_sub.write(css);
    Ok(())
}

// TODO
// delete all files
fn remove_missing() {}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}
