use anyhow::{anyhow, Context, Result};
use horrorshow::helper::doctype;
use horrorshow::owned_html;
use horrorshow::prelude::*;
use horrorshow::Template;
use std::io::BufWriter;
use std::path::Path;
use std::str;

#[macro_use]
extern crate horrorshow;

use mailparse::*;
use sha3::{
    digest::{ExtendableOutput, Update, XofReader},
    Shake128,
};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use urlencoding;

use config::{Config, INSTANCE};
mod config;
mod mbox;
mod time;
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
    date: u64, // unix epoch. received date (if present)
    date_string: String,
    body: String,
    mime: String,
}

#[derive(Debug, Clone)]
struct MailThread<'a> {
    messages: Vec<&'a Email>, // sorted
    hash: String,
    last_reply: u64,
}

impl<'a> MailThread<'a> {}
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
        None => &s.addr,
    }
}

impl<'a> ThreadList<'a> {
    fn write_atom_feed(&self) -> Result<()> {
        // TODO dry
        // not sure how well this feed works... it just tracks thread updates.
        let mut entries: String = String::new();
        for thread in &self.threads {
            let root = thread.messages[0];
            let tmpl = format!(
                r#"<title>{title}</title>
<link href="{item_link}"/>
<id>{entry_id}</id>
<updated>{updated_at}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
</feed>
"#,
                title = root.subject,
                item_link = "tbd",
                entry_id = "tbd",
                updated_at = "tbd",
                author_name = short_name(&root.from),
                author_email = &root.from.addr,
            );
            entries.push_str(&tmpl);
        }
        let atom = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>{feed_title}</title>
<link href="{feed_link}"/>
<updated>{last_updated}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
<id>{feed_id}</id>
<feed>
<entries>
{entry_list}
</entries>
</feed>"#,
            feed_title = Config::global().list_name,
            feed_link = Config::global().url,
            last_updated = "tbd",
            author_name = Config::global().list_email,
            author_email = Config::global().list_email,
            feed_id = "tbd",
            entry_list = entries,
        );
        let path = Config::global().out_dir.join("atom.xml");
        let mut file = File::create(&path)?;
        file.write(atom.as_bytes())?;
        Ok(())
    }
    pub fn write_to_file(&self) -> Result<()> {
        let tmp = html! {
            h1(class="page-title"): &Config::global().list_name;

            a(href=format!("mailto:{}", &Config::global().list_email)) {
                : &Config::global().list_email
            }
            span { // Hack
                : " | "
            }
            a(href=&Config::global().homepage) {
                : "about"
            }
            hr;
            @ for thread in &self.threads {
                div(class="message-sum") {
                    a(class="threadlink", href=format!("threads/{}.html", &thread.hash)) {
                        : &thread.messages[0].subject
                    }
                    br;
                    a(class="addr", href=format!("mailto:/{}", &thread.messages[0].from.addr)){
                        : short_name(&thread.messages[0].from)
                    }

                span(class="timeago") {
                    : format!(" | {} replies | {}", thread.messages.len() - 1, time::secs_to_date(thread.last_reply()).ymd())
                }
                }
            }
        };

        let file = File::create(&Config::global().out_dir.join("index.html"))?;
        let mut br = BufWriter::new(file);
        layout(Config::global().list_name.as_str(), tmp).write_to_io(&mut br)?;
        Ok(())
    }
}

impl<'a> MailThread<'a> {
    pub fn last_reply(&self) -> u64 {
        return self.messages[self.messages.len() - 1].date;
    }

    fn write_atom_feed(&self) -> Result<()> {
        // TODO dry
        let mut entries: String = String::new();
        for message in &self.messages {
            let tmpl = format!(
                r#"<title>{title}</title>
<link href="{item_link}"/>
<id>{entry_id}</id>
<updated>{updated_at}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
<content>
{content}
</content>
</feed>
"#,
                title = message.subject,
                item_link = "tbd",
                entry_id = "tbd",
                updated_at = "tbd",
                author_name = short_name(&message.from),
                author_email = &message.from.addr,
                content = &message.body,
            );
            entries.push_str(&tmpl);
        }
        let root = self.messages[0];
        let atom = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>{feed_title}</title>
<link href="{feed_link}"/>
<updated>{last_updated}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
<id>{feed_id}</id>
<feed>
<entries>
{entry_list}
</entries>
</feed>"#,
            feed_title = root.subject,
            feed_link = "tbd",
            last_updated = "tbd",
            author_name = short_name(&root.from),
            author_email = &root.from.addr,
            feed_id = "tbd",
            entry_list = entries,
        );
        let thread_dir = Config::global().out_dir.join("threads");
        let mut file = File::create(&thread_dir.join(format!("{}.xml", &self.hash)))?;
        file.write(atom.as_bytes())?;
        Ok(())
    }

    fn write_to_file(&self) -> Result<()> {
        let root = self.messages[0];
        let tmp = html! {
            h1(class="page-title"): &root.subject;
               div {
                a(href="../") {
                    : &Config::global().list_name
                }
              }     div {
                @ for message in &self.messages {
                    hr;
                    div(id=&message.id, class="message") {
                   span(class="bold") {
                        : &message.subject
                   }
                    @ if message.in_reply_to.is_some() { // TODO figure out match
                        a(title="replies-to", href=format!("#{}", message.in_reply_to.clone().unwrap())){
                            : " ^ "
                        }
                    }
                    br;
                   a(href=format!("mailto:{}", &message.from.addr), class="addr bold") {
                            : &message.from.to_string();
                        }
                        br;
                    span(class="timeago") {
                        : &message.date_string
                    }
                    a(title="permalink", href=format!("#{}", &message.id)) {
                        : " 🔗" 
                    }
                                        br; br;
                    div(class="email-body") {
                        : Raw(utils::email_body(&message.body))
                    }
                    br;
                    div(class="bold"){
                        a (href=message.mailto()) {
                            :"✉️ reply"
                        }
                    }
                    }
                }
            }
        };
        let thread_dir = Config::global().out_dir.join("threads");
        std::fs::create_dir(&thread_dir).ok();

        let file = File::create(&thread_dir.join(format!("{}.html", &self.hash)))?;
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
        // TODO verify encoding looks good and use percent_encoding instead
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
    let date_string = headers.get_first_value("date").context("No date header")?;

    // TODO use received.
    let date = dateparse(&date_string)? as u64;
    let from = addrparse_header(headers.get_first_header("from").context("No from header")?)?
        .extract_single_info()
        .context("Could not parse from header")?;

    return Ok(Email {
        id,
        in_reply_to,
        from,
        subject,
        date,
        date_string,
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

    let mut config = Config::from_file(&config_file).unwrap(); // TODO better err handling
    config.out_dir = out_dir.to_owned();
    INSTANCE.set(config).unwrap();

    let mbox = mbox::from_file(&in_mbox)?;

    let mut thread_index: HashMap<String, Vec<String>> = HashMap::new();

    let mut email_index: HashMap<String, Email> = HashMap::new();
    for entry in mbox {
        let buffer = entry.unwrap();
        let email = match local_parse_email(&buffer) {
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
    let thread_dir = Config::global().out_dir.join("threads");
    std::fs::create_dir(&thread_dir).ok();
    let mut threads = vec![];
    let mut curr_threads = get_current_threads(&out_dir);

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

        thread.write_to_file()?;
        thread.write_atom_feed()?;
        curr_threads.remove(&thread.hash);
        threads.push(thread);
    }

    for leftover in curr_threads {
        let file_to_remove = out_dir.join("threads").join(format!("{}.html", leftover));
        std::fs::remove_file(&file_to_remove)?;
    }

    // Remove any threads left over

    threads.sort_by_key(|a| a.last_reply);
    threads.reverse();
    let list = ThreadList { threads };
    list.write_to_file()?;
    list.write_atom_feed()?;
    // kinda clunky
    let css = include_bytes!("style.css");
    let mut css_root = File::create(out_dir.join("style.css"))?;
    css_root.write(css)?;
    let mut css_sub = File::create(out_dir.join("threads").join("style.css"))?;
    css_sub.write(css)?;
    Ok(())
}

// Use the sha3 hash of the ID. It is what it is.
// lots of unwrapping here
fn get_current_threads(out_dir: &Path) -> HashSet<String> {
    std::fs::read_dir(out_dir.join("threads"))
        .unwrap()
        .map(|x| {
            x.unwrap()
                .path()
                .file_stem()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned()
        })
        .filter(|x| !(x == "style"))
        .collect()
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}
