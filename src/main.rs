use anyhow::{anyhow, Context, Result};
use askama::Template;
use hex;
use mailparse::*;
use mbox_reader::MboxFile;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::prelude::*;
use urlencoding;

use config::{Config, INSTANCE};
mod config;
mod filters;

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

impl<'a> MailThread<'a> {
    pub fn last_reply(&self) -> u64 {
        return self.messages[self.messages.len() - 1].date;
    }

    fn build_atom_feed() -> String {
        String::new()
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
        pushencode("subject", &self.subject); // TODO Re:
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
        return hex::encode(&res1);
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
    let date = dateparse(
        &headers
            .get_first_value("received")
            .unwrap_or(headers.get_first_value("date").context("No date header")?),
    )? as u64;
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
    let thread_dir = &out_dir.join("threads");
    std::fs::create_dir(thread_dir).ok();

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

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(thread_dir.join(format!("{}.html", thread.hash)))?;
        file.write(
            Thread {
                thread: &thread,
                config: Config::global(),
            }
            .render()?
            .as_bytes(),
        )
        .ok();

        threads.push(thread);
    }

    threads.sort_by_key(|a| a.last_reply);
    threads.reverse();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(out_dir.join("index.html"))?;
    file.write(
        ThreadList {
            threads: threads,
            config: Config::global(),
        }
        .render()?
        .as_bytes(),
    )
    .ok();
    // kinda clunky
    std::fs::copy("templates/static/style.css", out_dir.join("style.css"))?;
    std::fs::copy("templates/static/style.css", thread_dir.join("style.css"))?;
    Ok(())
}

// TODO
// delete all files
fn remove_missing() {}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

#[derive(Template)]
#[template(path = "thread.html")]
struct Thread<'a> {
    thread: &'a MailThread<'a>,
    config: &'a Config,
}

#[derive(Template)]
#[template(path = "threadlist.html")]
struct ThreadList<'a> {
    // message root
    threads: Vec<MailThread<'a>>,
    config: &'a Config, // Not ideal repetition
}
