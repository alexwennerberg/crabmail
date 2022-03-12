// this code is not good
// i am not very good at rust
// that is ok though
#[forbid(unsafe_code)]
use anyhow::{Context, Result};
use maildir::Maildir;
use std::path::PathBuf;
use std::str;

use models::*;
use std::io::prelude::*;

use config::{Config, INSTANCE};
mod arg;
mod config;
mod maildir;
mod models;
mod templates;
mod threading;
mod time;
mod utils;

const ATOM_ENTRY_LIMIT: i32 = 100;
const PAGE_SIZE: i32 = 100;

// TODO

impl Lists<'_> {
    fn add(&mut self, list: threading::ThreadIdx) {
        // let newlist = List { threads: vec![] };
        for thread in list.threads {
            // let ids = thread.iter().map(|m| m.id).collect();
            // newlist.threads.push(Thread::from_id_list(ids));
        }
        // TODO sort threads
        // self.lists.push(newlist);
    }

    fn write_lists(&self) {
        std::fs::create_dir_all(&self.out_dir);
        let css = include_bytes!("style.css");
        write_if_unchanged(&self.out_dir.join("style.css"), css);
        let base_path = self.out_dir.join("index");
        write_if_unchanged(&base_path.with_extension("html"), self.to_html().as_bytes());
        if Config::global().include_gemini {
            write_if_unchanged(&base_path.with_extension("gmi"), self.to_gmi().as_bytes());
        }
        for list in &self.lists {
            list.write_all_files()
        }
    }
}
use std::fs::{read, write};

// TODO: use checksum / cache. bool whether it writes
fn write_if_unchanged(path: &PathBuf, data: &[u8]) -> bool {
    if let Ok(d) = read(path) {
        if &d == data {
            return false;
        }
    }
    write(path, data).unwrap();
    return true;
}

// / is disallowed in paths. ; is disallowed in message IDs
// assumes unix-like filesystem. TODO windows compatability if someone asks
fn pathescape_msg_id(s: &str) -> PathBuf {
    PathBuf::from(s.replace("/", ";"))
}

enum Format {
    XML,
    HTML,
    GMI,
}

impl List<'_> {
    // TODO move to main
    // fn from_maildir() -> Self { // TODO figure out init
    // where to live
    // List { threads: vec![] }
    fn write_all_files(&self) {
        let index = self.out_dir.join("index.html");
        // TODO write index (paginated) gmi
        // write index (paginated) html
        // write xml feed
        // Delete threads that aren't in my list (xml, gmi, html)
        for thread in &self.threads {
            let basepath = self
                .out_dir
                .join("threads")
                .join(&pathescape_msg_id(&thread.messages[0].id));
            // TODO cleanup, abstract
            write_if_unchanged(
                &basepath.with_extension("html"),
                thread.to_html().as_bytes(),
            );
            write_if_unchanged(&basepath.with_extension("xml"), thread.to_xml().as_bytes());
            if Config::global().include_gemini {
                write_if_unchanged(&basepath.with_extension("gmi"), thread.to_gmi().as_bytes());
            }
            // Delete nonexistent messages (cache?)
            // for file in thread
            // write raw file
        }
    }
}

fn main() -> Result<()> {
    let args = arg::Args::from_env();
    let maildir = &args.positional[0];
    let mut config = Config::from_file(&args.config)?;
    // TODO cleanup
    config.include_gemini = args.include_gemini;
    INSTANCE.set(config).unwrap();

    let mut lists = Lists {
        lists: vec![],
        out_dir: args.out_dir,
    };
    for maildir in std::fs::read_dir(maildir)?.filter_map(|m| m.ok()) {
        let dir_name = maildir.file_name().into_string().unwrap(); // TODO no unwrap
        if dir_name.as_bytes()[0] == b'.' || ["cur", "new", "tmp"].contains(&dir_name.as_str()) {
            continue;
        }

        let mut list = threading::ThreadIdx::new();
        let dirreader = Maildir::from(maildir.path());
        for f in dirreader
            .list_cur()
            .chain(dirreader.list_new())
            .filter_map(|e| e.ok())
        {
            let data = std::fs::read(f.path())?;
            // TODO move these 2 lines to dirreader
            let msg = mail_parser::Message::parse(&data).context("Missing mail bytes")?;
            list.add_email(&msg, f.path().to_path_buf());
        }
        list.finalize();
        lists.add(list);
    }

    lists.write_lists();
    Ok(())
}

// // Not a "raw email" struct, but an email object that can be represented by
// // crabmail.
// #[derive(Debug, Clone)]
// struct Email {
//     id: String,
//     from: SingleInfo,
//     subject: String,
//     in_reply_to: Option<String>,
//     date: u64, // unix epoch. received date (if present)
//     date_string: String,
//     body: String,
//     mime: String,
// }

// #[derive(Debug, Clone)]
// struct MailThread<'a> {
//     messages: Vec<&'a Email>, // sorted
//     hash: String,
//     last_reply: u64,
//     list_name: String,
// }

// impl<'a> MailThread<'a> {}

// fn layout(page_title: impl Render, content: impl Render) -> impl Render {
//     // owned_html _moves_ the arguments into the template. Useful for returning
//     // owned (movable) templates.
//     owned_html! {
//             : doctype::HTML;
//             html {
//             head {
//                 title : &page_title;
//                 : Raw("<meta http-equiv='Permissions-Policy' content='interest-cohort=()'/>
//                         <link rel='stylesheet' type='text/css' href='style.css' />
//                         <meta name='viewport' content='width=device-width, initial-scale=1.0, maximum-scale=1.0,user-scalable=0' />
//                         <link rel='icon' href='data:image/svg+xml,<svg xmlns=%22http://www.w3.org/2000/svg%22 viewBox=%220 0 100 100%22><text y=%22.9em%22 font-size=%2290%22>📧</text></svg>'>");
//                 meta(name="description", content=&page_title);
//             }
//             body {
//                 main {
//                 :&content
//                 }
//                 hr;
//             div(class="footer") {
//     : Raw("Archive generated with  <a href='https://crabmail.flounder.online/'>crabmail</a> at ");
//         : &Config::global().now;
//             }
//             }
//             }
//         }
// }

// struct ThreadList<'a> {
//     threads: Vec<MailThread<'a>>,
//     name: String,
//     email: String,
//     description: String,
//     title: String,
//     url: String, // URL?
// }

// // Get short name from an address name like "alex wennerberg <alex@asdfasdfafd>"
// fn short_name(s: &SingleInfo) -> &str {
//     match &s.display_name {
//         Some(dn) => dn,
//         None => &s.addr,
//     }
// }

// impl<'a> ThreadList<'a> {
//     fn new(threads: Vec<MailThread<'a>>, list_name: &str) -> Self {
//         let config = Config::global();
//         let d = config.default_subsection(&list_name);
//         let subsection_config = config
//             .subsections
//             .iter()
//             .find(|s| s.name == list_name)
//             .unwrap_or(&d);

//         ThreadList {
//             threads,
//             name: list_name.to_owned(), // TODO handle ownership
//             email: subsection_config.email.to_owned(),
//             title: subsection_config.title.to_owned(),
//             description: subsection_config.description.to_owned(),
//             url: format!("{}/{}", Config::global().base_url, &list_name),
//         }
//     }

//     pub fn write_to_file(&self) -> Result<()> {
//         let timestring = match Config::global().relative_times {
//             false => |t| time::secs_to_date(t).ymd(),
//             true => |t| time::timeago(t),
//         };
//         let tmp = html! {
//                     h1(class="page-title") {
//                         : &self.title;
//                         : Raw(" ");
//                         a(href="atom.xml") {
//                             // img(alt="Atom feed", src=utils::RSS_SVG);
//                         }
//                     }
//                     : Raw(&self.description);

//                     @if self.description.len() > 1 {
//                     br;
//                     }
//                     a(href=format!("mailto:{}", &self.email)) {
//                         : &self.email
//                     }
//                     hr;
//                     @ for thread in &self.threads {
//                         div(class="message-sum") {
//                             a(class="bigger", href=format!("threads/{}.html", &thread.hash)) {
//                                 : &thread.messages[0].subject
//                             }
//                             : format!(" ({})", thread.messages.len() -1) ;
//                             br;
//         span {
//         : short_name(&thread.messages[0].from)
//         }

//                        span(class="light") {
//                             : format!(" {created} | updated {last}",  created=timestring(thread.messages[0].date), last=timestring(thread.last_reply()))
//                         }                     br;

//                         }
//                     }
//                 };

//         let file = File::create(&Config::global().out_dir.join(&self.name).join("index.html"))?;
//         let mut br = BufWriter::new(file);
//         layout(self.name.clone(), tmp).write_to_io(&mut br)?;
//         Ok(())
//     }
// }

// impl<'a> MailThread<'a> {
//     pub fn last_reply(&self) -> u64 {
//         return self.messages[self.messages.len() - 1].date;
//     }

//     fn url(&self) -> String {
//         format!(
//             "{}/{}/threads/{}.html",
//             Config::global().base_url,
//             self.list_name,
//             self.hash
//         )
//     }

//     fn write_to_file(&self) -> Result<()> {
//         let root = self.messages[0];
//         let tmp = html! {
//                     h1(class="page-title") {
//                         : &root.subject;
//                         : Raw(" ");
//                         a(href=format!("./{}.xml", self.hash)) {
//                             // img(alt="Atom feed", src=utils::RSS_SVG);
//                         }
//                     }
//                        div {
//                         a(href="../") {
//                             : "Back";
//                         }
//                         : " ";
//                         a(href="#bottom") {
//                             : "Latest";
//                         }
//                         hr;
//                       }     div {
//                         @ for message in self.messages.iter() {
//                             div(id=&message.id, class="message") {
//                             div(class="message-meta") {
//                            span(class="bold") {
//                                 : &message.subject
//                            }

//                             br;
//                            a(href=format!("mailto:{}", &message.from.addr), class="bold") {
//                                     : &message.from.to_string();
//                                 }
//                                 br;
//                             span(class="light") {
//                                 : &message.date_string
//                             }
//                             a(title="permalink", href=format!("#{}", &message.id)) {
//                                 : " 🔗" 
//                             }
//                             @ if &message.mime == "text/html" {
//                                 span(class="light italic") {
//                                 : " (converted from html)";
//                                 }
//                             }
//                             br;
//                             a (class="bold", href=message.mailto(&self)) {
//                                 :"✉️ Reply"
//                             }
//                             @ if Config::global().include_raw {
//                                : " [";
//                                a(href=format!("../messages/{}", message.id)) {
//                                    : "Download" ;
//                                }
//                                : "]";
//                            }
//                             @ if message.in_reply_to.is_some() {
//                                 : " ";
//                                 a(title="replies-to", href=format!("#{}", message.in_reply_to.clone().unwrap())){
//                                     : "Parent";
//                                 }
//         }
//                             }
//                             br;
//                             @ if message.subject.starts_with("[PATCH") ||  message.subject.starts_with("[PULL") {
//                                 div(class="email-body monospace") {
//                                     // : Raw(utils::email_body(&message.body))
//                                 }
//                             } else {
//                                 div(class="email-body") {
//                                     // : Raw(utils::email_body(&message.body))
//                                 }
//                             } br;
//                             }
//                         }
//                         a(id="bottom");
//                     }
//                 };
//         let thread_dir = Config::global()
//             .out_dir
//             .join(&self.list_name)
//             .join("threads");
//         std::fs::create_dir(&thread_dir).ok();

//         let file = File::create(&thread_dir.join(format!("{}.html", &self.hash)))?;
//         let mut br = BufWriter::new(file);
//         layout(root.subject.as_str(), tmp).write_to_io(&mut br)?;
//         Ok(())
//     }
// }

// impl Email {
//     // mailto:... populated with everything you need
//     // TODO add these to constructors
//     pub fn url(&self, thread: &MailThread) -> String {
//         format!("{}#{}", thread.url(), self.id)
//     }
//     pub fn mailto(&self, thread: &MailThread) -> String {
//         let config = Config::global();
//         let d = config.default_subsection(&thread.list_name);
//         let subsection_config = config
//             .subsections
//             .iter()
//             .find(|s| s.name == thread.list_name)
//             .unwrap_or(&d);

//         let mut url = format!("mailto:{}?", subsection_config.email);

//         let from = self.from.to_string();
//         // make sure k is already urlencoded
//         let mut pushencode = |k: &str, v| {
//             url.push_str(&format!("{}={}&", k, urlencoding::encode(v)));
//         };
//         let fixed_id = format!("<{}>", &self.id);
//         pushencode("cc", &from);
//         pushencode("in-reply-to", &fixed_id);
//         let list_url = format!("{}/{}", &Config::global().base_url, &thread.list_name);
//         pushencode("list-archive", &list_url);
//         pushencode("subject", &format!("Re: {}", thread.messages[0].subject));
//         // quoted body
//         url.push_str("body=");
//         // This is ugly and I dont like it. May deprecate it
//         if Config::global().reply_add_link {
//             url.push_str(&format!(
//                 "[View original message: {}]%0A%0A",
//                 &urlencoding::encode(&thread.url())
//             ));
//         }
//         for line in self.body.lines() {
//             url.push_str("%3E%20");
//             url.push_str(&urlencoding::encode(&line));
//             url.push_str("%0A");
//         }
//         url.into()
//     }

//     // TODO rename
//     pub fn hash(&self) -> String {
//         self.id.replace("/", ";")
//     }
// }

// const EXPORT_HEADERS: &[&str] = &[
//     "Date",
//     "Subject",
//     "From",
//     "Sender",
//     "Reply-To",
//     "To",
//     "Cc",
//     "Bcc",
//     "Message-Id",
//     "In-Reply-To",
//     "References",
//     "MIME-Version",
//     "Content-Type",
//     "Content-Disposition",
//     "Content-Transfer-Encoding",
// ];

// fn write_parsed_mail(parsed_mail: &ParsedMail, f: &mut std::fs::File) -> Result<()> {
//     for header in parsed_mail.get_headers() {
//         // binary search?
//         if EXPORT_HEADERS.contains(&header.get_key().as_str()) {
//             f.write_all(header.get_key_raw())?;
//             f.write_all(b": ")?;
//             f.write_all(header.get_value_raw())?;
//             f.write_all(b"\r\n")?;
//         }
//     }
//     f.write_all(b"\r\n")?;
//     f.write_all(&parsed_mail.get_body_raw()?)?;
//     Ok(())
// }

// fn local_parse_email(parsed_mail: &ParsedMail) -> Result<Email> {
//     let mut body: String = "[Message has no body]".to_owned();
//     let mut mime: String = "".to_owned();
//     let nobody = "[No body found]";
//     // nested lookup
//     let mut queue = vec![parsed_mail];
//     let mut text_body = None;
//     let mut html_body = None;
//     while queue.len() > 0 {
//         let top = queue.pop().unwrap();
//         for sub in &top.subparts {
//             queue.push(sub);
//         }
//         let content_disposition = top.get_content_disposition();
//         if content_disposition.disposition == mailparse::DispositionType::Attachment {
//             // attachment handler
//         } else {
//             if top.ctype.mimetype == "text/plain" {
//                 let b = top.get_body().unwrap_or(nobody.to_owned());
//                 if parsed_mail.ctype.params.get("format") == Some(&"flowed".to_owned()) {
//                     text_body = Some(b);
//                     // text_body = Some(utils::unformat_flowed(&b));
//                 } else {
//                     text_body = Some(b);
//                 }
//             }
//             if top.ctype.mimetype == "text/html" {
//                 html_body = Some(nanohtml2text::html2text(
//                     &top.get_body().unwrap_or(nobody.to_owned()),
//                 ));
//             }
//         }
//     }
//     if let Some(b) = text_body {
//         body = b;
//         mime = "text/plain".to_owned();
//     } else if let Some(b) = html_body {
//         body = b;
//         mime = "text/html".to_owned();
//     }
//     let headers = &parsed_mail.headers;
//     let id = headers
//         .get_first_value("message-id")
//         .and_then(|m| {
//             msgidparse(&m).ok().and_then(|i| match i.len() {
//                 0 => None,
//                 _ => Some(i[0].clone()),
//             })
//         })
//         .context("No valid message ID")?;
//     // Assume 1 in-reply-to header. a reasonable assumption
//     let in_reply_to = headers.get_first_value("in-reply-to").and_then(|m| {
//         msgidparse(&m).ok().and_then(|i| match i.len() {
//             0 => None,
//             _ => Some(i[0].clone()),
//         })
//     });
//     let subject = headers
//         .get_first_value("subject")
//         .unwrap_or("(no subject)".to_owned());
//     // TODO move upstream, add key/value parsing
//     // https://datatracker.ietf.org/doc/html/rfc2822.html#section-3.6.7
//     // https://github.com/staktrace/mailparse/issues/99
//     let received = headers.get_first_value("received");
//     let date_string = match received {
//         Some(r) => {
//             let s: Vec<&str> = r.split(";").collect();
//             s[s.len() - 1].to_owned()
//         }
//         None => headers.get_first_value("date").context("No date header")?,
//     };

//     let date = dateparse(&date_string)? as u64;
//     let from = addrparse_header(headers.get_first_header("from").context("No from header")?)?
//         .extract_single_info()
//         .context("Could not parse from header")?;

//     return Ok(Email {
//         id,
//         in_reply_to,
//         from,
//         subject,
//         date,
//         date_string,
//         body,
//         mime,
//     });
// }

// // if [arg] has cur,new,tmp -> that is the index
// // else, do each subfolder

// fn write_index(lists: Vec<String>) -> Result<()> {
//     let description = &Config::global().description;
//     let tmp = html! {
//     h1(class="page-title") {
//         : format!("Mail Archives");
//     }
//     : Raw(&description);

//         @if description.len() > 1 {
//         br;
//     }
//      hr;
//     @for list in &lists {
//         a(href=list, class="bigger bold") {
//             :list;
//         }
//         br;
//     }
//     };
//     let file = File::create(&Config::global().out_dir.join("index.html"))?;
//     let mut br = BufWriter::new(file);
//     layout("Mail Archives".to_string(), tmp).write_to_io(&mut br)?;
//     Ok(())
// }

// // fn oldmain() -> Result<()> {
// //     let args = arg::Args::from_env();

// //     let mut config = Config::from_file(&args.config)?;
// //     config.out_dir = args.out_dir;
// //     config.relative_times = args.flags.contains('r');
// //     config.include_raw = args.flags.contains('R');
// //     INSTANCE.set(config).unwrap();

// //     // let is_subfolder = std::fs::read_dir(&args.maildir)
// //     // .unwrap()
// //     // .any(|a| a.unwrap().file_name().to_str().unwrap() == "cur");

// //     let css = include_bytes!("style.css");
// //     let mut names = vec![];
// //     let mut message_count = 0;
// //     for maildir in std::fs::read_dir(&args.maildir).unwrap() {
// //         let maildir = maildir?;
// //         let file_name = maildir.file_name();
// //         let config = Config::global();
// //         let out_dir = config.out_dir.join(&file_name);
// //         std::fs::create_dir(&out_dir).ok();
// //         let dirreader = Maildir::from(maildir.path().to_str().unwrap());
// //         let list_name = file_name.into_string().unwrap();
// //         // filter out maildir internal junk
// //         if list_name.as_bytes()[0] == b'.' || ["cur", "new", "tmp"].contains(&list_name.as_str()) {
// //             continue;
// //         }

// //         let path = out_dir.join("messages");
// //         std::fs::remove_dir_all(&path).ok();
// //         names.push(list_name.clone());
// //         // new world WIP
// //         // let mut threader = threading::Arena::default();
// //         // Loads whole file into memory for threading
// //         // for item in maildir.list_cur().chain(maildir.list_new()) {
// //         // threader.add_message(item?);
// //         // }
// //         // threader.finalize();
// //         // return Ok(());

// //         let mut thread_index: HashMap<String, Vec<String>> = HashMap::new();

// //         let mut email_index: HashMap<String, Email> = HashMap::new();
// //         for entry in dirreader.list_cur().chain(dirreader.list_new()) {
// //             let mut tmp = entry.unwrap();
// //             let buffer = tmp.parsed()?;
// //             // persist raw messages
// //             let email = match local_parse_email(&buffer) {
// //                 Ok(e) => e,
// //                 Err(e) => {
// //                     eprintln!("Error parsing {:?} -- {:?}", tmp.path(), e);
// //                     continue;
// //                 }
// //             };
// //             message_count += 1;
// //             // write raw emails
// //             if Config::global().include_raw {
// //                 // inefficient here -- no diff
// //                 std::fs::create_dir(&path).ok();
// //                 let mut file = File::create(out_dir.join("messages").join(email.hash()))?;
// //                 write_parsed_mail(&buffer, &mut file)?;
// //             }
// //             // TODO fix borrow checker
// //             if let Some(reply) = email.in_reply_to.clone() {
// //                 match thread_index.get(&reply) {
// //                     Some(_) => {
// //                         let d = thread_index.get_mut(&reply).unwrap();
// //                         d.push(email.id.clone());
// //                     }
// //                     None => {
// //                         thread_index.insert(reply, vec![email.id.clone()]);
// //                     }
// //                 }
// //             }
// //             email_index.insert(email.id.clone(), email);
// //         }

// //         // Add index by subject lines
// //         // atrocious
// //         let mut todo = vec![]; // im bad at borrow checker
// //         for (_, em) in &email_index {
// //             if em.in_reply_to.is_none()
// //                 && (em.subject.starts_with("Re: ") || em.subject.starts_with("RE: "))
// //             {
// //                 // TODO O(n^2)
// //                 for (_, em2) in &email_index {
// //                     if em2.subject == em.subject[4..] {
// //                         match thread_index.get(&em2.id) {
// //                             Some(_) => {
// //                                 let d = thread_index.get_mut(&em2.id).unwrap();
// //                                 d.push(em.id.clone());
// //                             }
// //                             None => {
// //                                 thread_index.insert(em2.id.clone(), vec![em.id.clone()]);
// //                             }
// //                         }
// //                         todo.push((em.id.clone(), em2.id.clone()));
// //                         break;
// //                     }
// //                 }
// //             }
// //         }
// //         for (id, reply) in todo {
// //             let em = email_index.get_mut(&id).unwrap();
// //             em.in_reply_to = Some(reply)
// //         }

// //         let mut thread_roots: Vec<Email> = email_index
// //             .iter()
// //             .filter_map(|(_, v)| {
// //                 if v.in_reply_to.is_none() {
// //                     // or can't find root based on Re: subject
// //                     return Some(v.clone());
// //                 }
// //                 return None;
// //             })
// //             .collect();
// //         let thread_dir = out_dir.join("threads");
// //         std::fs::create_dir_all(&thread_dir).ok();
// //         let mut threads = vec![];
// //         let mut curr_threads = get_current_threads(&thread_dir);

// //         for root in &mut thread_roots {
// //             let mut thread_ids = vec![];
// //             let mut current: Vec<String> = vec![root.id.clone()];
// //             while current.len() > 0 {
// //                 let top = current.pop().unwrap().clone();
// //                 thread_ids.push(top.clone());
// //                 if let Some(ids) = thread_index.get(&top.clone()) {
// //                     for item in ids {
// //                         current.push(item.to_string());
// //                     }
// //                 }
// //             }

// //             let mut messages: Vec<&Email> = thread_ids
// //                 .iter()
// //                 .map(|id| email_index.get(id).unwrap())
// //                 .collect();

// //             messages.sort_by_key(|a| a.date);

// //             let mut thread = MailThread {
// //                 messages: messages,
// //                 hash: root.hash(),
// //                 last_reply: 0, // TODO
// //                 list_name: list_name.clone(),
// //             };

// //             thread.last_reply = thread.last_reply();

// //             thread.write_to_file()?;
// //             thread.write_atom_feed()?;
// //             curr_threads.remove(&thread.hash);
// //             threads.push(thread);
// //         }

// //         for leftover in curr_threads {
// //             let file_to_remove = thread_dir.join(format!("{}.html", leftover));
// //             std::fs::remove_file(&file_to_remove).ok();
// //             let file_to_remove = thread_dir.join(format!("{}.xml", leftover));
// //             std::fs::remove_file(&file_to_remove).ok();
// //         }

// //         // Remove any threads left over

// //         threads.sort_by_key(|a| a.last_reply);
// //         threads.reverse();
// //         let list = ThreadList::new(threads, &list_name);
// //         list.write_to_file()?;
// //         list.write_atom_feed()?;
// //         // kinda clunky
// //         let mut css_root = File::create(out_dir.join("style.css"))?;
// //         css_root.write(css)?;
// //         let mut css_sub = File::create(out_dir.join("threads").join("style.css"))?;
// //         css_sub.write(css)?;
// //     }
// //     let mut css_root = File::create(Config::global().out_dir.join("style.css"))?;
// //     css_root.write(css)?;
// //     write_index(names)?;
// //     eprintln!("Processed {} emails", message_count);
// //     Ok(())
// // }

// // Use the sha3 hash of the ID. It is what it is.
// // lots of unwrapping here
// fn get_current_threads(thread_dir: &Path) -> HashSet<String> {
//     std::fs::read_dir(thread_dir)
//         .unwrap()
//         .map(|x| {
//             x.unwrap()
//                 .path()
//                 .file_stem()
//                 .unwrap()
//                 .to_str()
//                 .unwrap()
//                 .to_owned()
//         })
//         .filter(|x| !(x == "style"))
//         .collect()
// }
