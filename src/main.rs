use anyhow::{anyhow, Context, Result};
use horrorshow::helper::doctype;
use horrorshow::owned_html;
use horrorshow::prelude::*;
use horrorshow::Template;
use maildir::Maildir;
use std::io::BufWriter;
use std::path::Path;
use std::str;

#[macro_use]
extern crate horrorshow;

use mailparse::*;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::prelude::*;
use urlencoding;

use config::{Config, INSTANCE};
use utils::xml_safe;
mod arg;
mod config;
mod time;
mod utils;

// TODO be more clear about the expected input types
// maildir

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
    list_name: String,
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
    : Raw("Archive generated with  <a href='https://crabmail.flounder.online/'>crabmail</a>")
            }
            }
            }
        }
}

struct ThreadList<'a> {
    threads: Vec<MailThread<'a>>,
    name: String,
    email: String,
    url: String, // URL?
}

// Get short name from an address name like "alex wennerberg <alex@asdfasdfafd>"
fn short_name(s: &SingleInfo) -> &str {
    match &s.display_name {
        Some(dn) => dn,
        None => &s.addr,
    }
}

impl<'a> ThreadList<'a> {
    fn new(threads: Vec<MailThread<'a>>, list_name: &str) -> Self {
        ThreadList {
            threads,
            name: list_name.to_owned(),
            email: Config::global().email_fmt.replace("%s", &list_name),
            url: format!("{}/{}", Config::global().base_url, &list_name),
        }
    }
    fn write_atom_feed(&self) -> Result<()> {
        // TODO dry
        // not sure how well this feed works... it just tracks thread updates.
        let mut entries: String = String::new();
        let mut last_updated = u64::MAX;
        for thread in &self.threads {
            let root = thread.messages[0];
            let tmpl = format!(
                r#"<entry>
<title>{title}</title>
<link href="{item_link}"/>
<id>{entry_id}</id>
<updated>{updated_at}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
</entry>
"#,
                title = xml_safe(&root.subject),
                item_link = thread.url(),
                entry_id = thread.url(),
                updated_at = time::secs_to_date(thread.last_reply).rfc3339(),
                author_name = xml_safe(short_name(&root.from)),
                author_email = xml_safe(&root.from.addr),
            );
            if thread.last_reply < last_updated {
                last_updated = thread.last_reply;
            }
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
{entry_list}
</feed>"#,
            feed_title = &self.name,
            feed_link = &self.url,
            last_updated = time::secs_to_date(last_updated).rfc3339(),
            author_name = &self.email,
            author_email = &self.email,
            feed_id = &self.url,
            entry_list = entries,
        );
        let path = Config::global().out_dir.join(&self.name).join("atom.xml");
        let mut file = File::create(&path)?;
        file.write(atom.as_bytes())?;
        Ok(())
    }
    pub fn write_to_file(&self) -> Result<()> {
        let timestring = match Config::global().relative_times {
            false => |t| time::secs_to_date(t).ymd(),
            true => |t| time::timeago(t),
        };
        let tmp = html! {
                    h1(class="page-title") {
                        : format!("{} Mailing List", &self.name);
                        : Raw(" ");
                        a(href="atom.xml") {
                            img(alt="Atom feed", src=utils::RSS_SVG);
                        }
                    }

                    a(href=format!("mailto:{}", &self.email)) {
                        : &self.email
                    }
                    hr;
                    @ for thread in &self.threads {
                        div(class="message-sum") {
                            a(class="bigger", href=format!("threads/{}.html", &thread.hash)) {
                                : &thread.messages[0].subject
                            }
                            : format!(" ({})", thread.messages.len() -1) ;
                            br;
        span {
        : short_name(&thread.messages[0].from)
        }

                       span(class="light") {
                            : format!(" {created} | updated {last}",  created=timestring(thread.messages[0].date), last=timestring(thread.last_reply()))
                        }                     br;

                        }
                    }
                };

        let file = File::create(&Config::global().out_dir.join(&self.name).join("index.html"))?;
        let mut br = BufWriter::new(file);
        layout(self.name.clone(), tmp).write_to_io(&mut br)?;
        Ok(())
    }
}

impl<'a> MailThread<'a> {
    pub fn last_reply(&self) -> u64 {
        return self.messages[self.messages.len() - 1].date;
    }

    fn url(&self) -> String {
        format!(
            "{}/{}/threads/{}.html",
            Config::global().base_url,
            self.list_name,
            self.hash
        )
    }

    fn write_atom_feed(&self) -> Result<()> {
        // TODO dry
        let mut entries: String = String::new();
        for message in &self.messages {
            let tmpl = format!(
                r#"<entry>
<title>{title}</title>
<link href="{item_link}"/>
<id>{entry_id}</id>
<updated>{updated_at}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
<content type="text/plain">
{content}
</content>
</entry>
"#,
                title = xml_safe(&message.subject),
                item_link = self.url(),
                entry_id = xml_safe(&format!("{}#{}", self.url(), message.id)),
                updated_at = time::secs_to_date(message.date).rfc3339(),
                author_name = xml_safe(short_name(&message.from)),
                author_email = xml_safe(&message.from.addr),
                content = xml_safe(&message.body),
            );
            entries.push_str(&tmpl);
        }
        let root = self.messages[0];
        let atom = format!(
            r#"<?xml version="1.0" encoding="utf-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
<title>{feed_title}</title>
<link rel="self" href="{feed_link}"/>
<updated>{last_updated}</updated>
<author>
    <name>{author_name}</name>
    <email>{author_email}</email>
</author>
<id>{feed_id}</id>
{entry_list}
</feed>"#,
            feed_title = xml_safe(&root.subject),
            feed_link = self.url(),
            last_updated = time::secs_to_date(self.last_reply()).rfc3339(),
            author_name = xml_safe(short_name(&root.from)),
            author_email = xml_safe(&root.from.addr),
            feed_id = self.url(),
            entry_list = entries,
        );
        let thread_dir = Config::global()
            .out_dir
            .join(&self.list_name)
            .join("threads");
        let mut file = File::create(&thread_dir.join(format!("{}.xml", &self.hash)))?;
        file.write(atom.as_bytes())?;
        Ok(())
    }

    fn write_to_file(&self) -> Result<()> {
        let root = self.messages[0];
        let tmp = html! {
            h1(class="page-title") {
                : &root.subject;
                : Raw(" ");
                a(href=format!("./{}.xml", self.hash)) {
                    img(alt="Atom feed", src=utils::RSS_SVG);
                }
            }
               div {
                a(href="../") {
                    : "Back";
                }
              }     div {
                @ for message in self.messages.iter() {
                    hr;
                    div(id=&message.id, class="message") {
                   span(class="bold") {
                        : &message.subject
                   }
                    @ if message.in_reply_to.is_some() { // TODO figure out match
                        : " ";
                        a(title="replies-to", href=format!("#{}", message.in_reply_to.clone().unwrap())){
                            : "In reply to ^";
                        }
                    }
                    br;
                   a(href=format!("mailto:{}", &message.from.addr), class="bold") {
                            : &message.from.to_string();
                        }
                        br;
                    span(class="light") {
                        : &message.date_string
                    }
                    a(title="permalink", href=format!("#{}", &message.id)) {
                        : " 🔗" 
                    }
                    @ if &message.mime == "text/html" {
                        span(class="light italic") {
                        : " (converted from html)";
                        }
                    }
                                        br; br;
                    div(class="email-body") {
                        : Raw(utils::email_body(&message.body))
                    }
                    br;
                    div(class="bold"){
                        a (href=message.mailto(&root.subject, &self.list_name)) {
                            :"✉️ Reply"
                        }
                    }
                    }
                }
            }
        };
        let thread_dir = Config::global()
            .out_dir
            .join(&self.list_name)
            .join("threads");
        std::fs::create_dir(&thread_dir).ok();

        let file = File::create(&thread_dir.join(format!("{}.html", &self.hash)))?;
        let mut br = BufWriter::new(file);
        layout(root.subject.as_str(), tmp).write_to_io(&mut br)?;
        Ok(())
    }
}

impl Email {
    // mailto:... populated with everything you need
    pub fn mailto(&self, thread_subject: &str, list_name: &str) -> String {
        // TODO configurable
        let mut url = format!(
            "mailto:{}?",
            Config::global().email_fmt.replace("%s", list_name) // not ideal
        );

        let from = self.from.to_string();
        // make sure k is already urlencoded
        let mut pushencode = |k: &str, v| {
            url.push_str(&format!("{}={}&", k, urlencoding::encode(v)));
        };
        // TODO verify encoding looks good and use percent_encoding instead
        let fixed_id = format!("<{}>", &self.id);
        pushencode("cc", &from);
        pushencode("in-reply-to", &fixed_id);
        pushencode("subject", &format!("Re: {}", thread_subject));
        // quoted body
        url.push_str("body=");
        for line in self.body.lines() {
            url.push_str("%3E%20");
            url.push_str(&urlencoding::encode(&line));
            url.push_str("%0A");
        }
        url.into()
    }

    // TODO rename
    //
    pub fn hash(&self) -> String {
        self.id.replace("/", ";")
    }
}

fn local_parse_email(parsed_mail: &ParsedMail) -> Result<Email> {
    let mut body: String = "[Message has no body]".to_owned();
    let mut mime: String = "".to_owned();
    let nobody = "[No body found]";
    // nested lookup
    let mut queue = vec![parsed_mail];
    let text_found = false;
    let mut text_body = None;
    let mut html_body = None;
    while queue.len() > 0 {
        let top = queue.pop().unwrap();
        for sub in &top.subparts {
            queue.push(sub);
        }
        let content_disposition = top.get_content_disposition();
        if content_disposition.disposition == mailparse::DispositionType::Attachment {
            // attachment handler
        } else {
            if top.ctype.mimetype == "text/plain" {
                let b = top.get_body().unwrap_or(nobody.to_owned());
                if parsed_mail.ctype.params.get("format") == Some(&"flowed".to_owned()) {
                    text_body = Some(utils::unformat_flowed(&b));
                } else {
                    text_body = Some(b);
                }
            }
            if top.ctype.mimetype == "text/html" {
                html_body = Some(nanohtml2text::html2text(
                    &top.get_body().unwrap_or(nobody.to_owned()),
                ));
            }
        }
    }
    if let Some(b) = text_body {
        body = b;
        mime = "text/plain".to_owned();
    } else if let Some(b) = html_body {
        body = b;
        mime = "text/html".to_owned();
    }
    let headers = &parsed_mail.headers;
    let id = headers
        .get_first_value("message-id")
        .and_then(|m| {
            msgidparse(&m).ok().and_then(|i| match i.len() {
                0 => None,
                _ => Some(i[0].clone()),
            })
        })
        .context("No valid message ID")?;
    if id.contains("..") {
        // dont hack me
        // id goes into filename. TODO more verification
        return Err(anyhow!("bad message ID"));
    }
    // Assume 1 in-reply-to header. a reasonable assumption
    let in_reply_to = headers.get_first_value("in-reply-to").and_then(|m| {
        msgidparse(&m).ok().and_then(|i| match i.len() {
            0 => None,
            _ => Some(i[0].clone()),
        })
    });
    let subject = headers
        .get_first_value("subject")
        .unwrap_or("(no subject)".to_owned());
    // TODO move upstream, add key/value parsing
    // https://datatracker.ietf.org/doc/html/rfc2822.html#section-3.6.7
    // https://github.com/staktrace/mailparse/issues/99
    let received = headers.get_first_value("received");
    let date_string = match received {
        Some(r) => {
            let s: Vec<&str> = r.split(";").collect();
            s[s.len() - 1].to_owned()
        }
        None => headers.get_first_value("date").context("No date header")?,
    };

    // TODO TODO
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

// if [arg] has cur,new,tmp -> that is the index
// else, do each subfolder

fn write_index(lists: Vec<String>) -> Result<()> {
    let tmp = html! {
    h1(class="page-title") {
        : format!("Mailing Lists");
    }
    hr;
    @for list in &lists {
        a(href=list, class="bigger bold") {
            :list;
        }
        br;
    }
    };
    let file = File::create(&Config::global().out_dir.join("index.html"))?;
    let mut br = BufWriter::new(file);
    layout("Mailing Lists".to_string(), tmp).write_to_io(&mut br)?;
    Ok(())
}

fn main() -> Result<()> {
    let args = arg::Args::from_env();

    let mut config = Config::from_file(&args.config).unwrap(); // TODO better err handling
    config.out_dir = args.out_dir;
    config.relative_times = args.flags.contains('r');
    INSTANCE.set(config).unwrap();

    // let is_subfolder = std::fs::read_dir(&args.maildir)
    // .unwrap()
    // .any(|a| a.unwrap().file_name().to_str().unwrap() == "cur");

    let css = include_bytes!("style.css");
    let mut names = vec![];
    for maildir in std::fs::read_dir(&args.maildir).unwrap() {
        let maildir = maildir?;
        let dirreader = Maildir::from(maildir.path().to_str().unwrap());
        let file_name = maildir.file_name();
        let out_dir = &Config::global().out_dir.join(&file_name);
        let list_name = file_name.into_string().unwrap();
        // filter out maildir internal junk
        if list_name.as_bytes()[0] == b'.' || ["cur", "new", "tmp"].contains(&list_name.as_str()) {
            continue;
        }
        names.push(list_name.clone());
        // new world WIP
        // let mut threader = threading::Arena::default();
        // Loads whole file into memory for threading
        // for item in maildir.list_cur().chain(maildir.list_new()) {
        // threader.add_message(item?);
        // }
        // threader.finalize();
        // return Ok(());

        let mut thread_index: HashMap<String, Vec<String>> = HashMap::new();

        let mut email_index: HashMap<String, Email> = HashMap::new();
        for entry in dirreader.list_cur().chain(dirreader.list_new()) {
            let mut tmp = entry.unwrap();
            let buffer = tmp.parsed()?;
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

        // Add index by subject lines
        // atrocious
        let mut todo = vec![]; // im bad at borrow checker
        for (_, em) in &email_index {
            if em.in_reply_to.is_none()
                && (em.subject.starts_with("Re: ") || em.subject.starts_with("RE: "))
            {
                // TODO O(n^2)
                for (_, em2) in &email_index {
                    if em2.subject == em.subject[4..] {
                        match thread_index.get(&em2.id) {
                            Some(_) => {
                                let d = thread_index.get_mut(&em2.id).unwrap();
                                d.push(em.id.clone());
                            }
                            None => {
                                thread_index.insert(em2.id.clone(), vec![em.id.clone()]);
                            }
                        }
                        todo.push((em.id.clone(), em2.id.clone()));
                        break;
                    }
                }
            }
        }
        for (id, reply) in todo {
            let em = email_index.get_mut(&id).unwrap();
            em.in_reply_to = Some(reply)
        }

        let mut thread_roots: Vec<Email> = email_index
            .iter()
            .filter_map(|(_, v)| {
                if v.in_reply_to.is_none() {
                    // or can't find root based on Re: subject
                    return Some(v.clone());
                }
                return None;
            })
            .collect();
        std::fs::create_dir(&out_dir).ok();
        let thread_dir = out_dir.join("threads");
        std::fs::create_dir_all(&thread_dir).ok();
        let mut threads = vec![];
        let mut curr_threads = get_current_threads(&thread_dir);

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
                list_name: list_name.clone(),
            };

            thread.last_reply = thread.last_reply();

            thread.write_to_file()?;
            thread.write_atom_feed()?;
            curr_threads.remove(&thread.hash);
            threads.push(thread);
        }

        for leftover in curr_threads {
            let file_to_remove = thread_dir.join(format!("{}.html", leftover));
            std::fs::remove_file(&file_to_remove).ok();
            let file_to_remove = thread_dir.join(format!("{}.xml", leftover));
            std::fs::remove_file(&file_to_remove).ok();
        }

        // Remove any threads left over

        threads.sort_by_key(|a| a.last_reply);
        threads.reverse();
        let list = ThreadList::new(threads, &list_name);
        list.write_to_file()?;
        list.write_atom_feed()?;
        // kinda clunky
        // TODO replace with symlinks
        let mut css_root = File::create(out_dir.join("style.css"))?;
        css_root.write(css)?;
        let mut css_sub = File::create(out_dir.join("threads").join("style.css"))?;
        css_sub.write(css)?;
    }
    let mut css_root = File::create(Config::global().out_dir.join("style.css"))?;
    css_root.write(css)?;
    write_index(names)?;
    Ok(())
}

// Use the sha3 hash of the ID. It is what it is.
// lots of unwrapping here
fn get_current_threads(thread_dir: &Path) -> HashSet<String> {
    std::fs::read_dir(thread_dir)
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
