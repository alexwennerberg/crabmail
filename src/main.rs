// this code is not good
// i am not very good at rust
// that is ok though
#[forbid(unsafe_code)]
use anyhow::{Context, Result};
use maildir::Maildir;
use std::path::PathBuf;
use std::str;
use time::Date;

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

const ATOM_ENTRY_LIMIT: i32 = 100;
const PAGE_SIZE: i32 = 100;

use std::ffi::{OsStr, OsString};
use std::path::Path;

// stole it from the internet
pub fn append_ext(ext: impl AsRef<OsStr>, path: &PathBuf) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(ext.as_ref());
    os_string.into()
}

impl Lists {
    fn write_lists(&mut self) {
        std::fs::create_dir_all(&self.out_dir);
        let css = include_bytes!("style.css");
        write_if_unchanged(&self.out_dir.join("style.css"), css);
        let base_path = self.out_dir.join("index");
        write_if_unchanged(&base_path.with_extension("html"), self.to_html().as_bytes());
        if Config::global().include_gemini {
            write_if_unchanged(&base_path.with_extension("gmi"), self.to_gmi().as_bytes());
        }
        for list in &mut self.lists {
            list.persist();
            // todo somewhat awkward
            write_if_unchanged(&list.out_dir.join("style.css"), css);
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

enum Format {
    XML,
    HTML,
    GMI,
}

impl List {
    fn persist(&mut self) {
        // let written = hashset
        self.write_index();
        self.write_threads();
        // for file in threads, messages
        // if not in written
        // delete
    }
    fn write_index(&self) {
        // TODO fix lazy copy paste
        // TODO return files written
        for (n, gmi) in self.to_gmi().iter().enumerate() {
            let index;
            if n == 0 {
                index = self.out_dir.join("index");
            } else {
                index = self.out_dir.join(format!("{}-{}", "index", n));
            }
            write_if_unchanged(&index.with_extension("gmi"), gmi.as_bytes());
        }
        for (n, html) in self.to_html().iter().enumerate() {
            let index;
            if n == 0 {
                index = self.out_dir.join("index");
            } else {
                index = self.out_dir.join(format!("{}-{}", "index", n));
            }
            write_if_unchanged(&index.with_extension("html"), html.as_bytes());
        }
        // write_if_unchanged(&self.out_dir.join("atom.xml"), self.to_xml().as_bytes());
    }

    fn write_threads(&mut self) {
        // files written = HashSet
        let thread_dir = self.out_dir.join("threads");
        let message_dir = self.out_dir.join("messages");
        std::fs::create_dir_all(&thread_dir).unwrap();
        std::fs::create_dir_all(&message_dir).unwrap();
        for thread_ids in &self.thread_idx.threads {
            // Load thread
            let thread = Thread::new(thread_ids);
            let basepath = thread_dir.join(&thread.messages[0].pathescape_msg_id());
            // hacky
            write_if_unchanged(&append_ext("html", &basepath), thread.to_html().as_bytes());
            write_if_unchanged(&append_ext("xml", &basepath), thread.to_xml().as_bytes());
            if Config::global().include_gemini {
                write_if_unchanged(&append_ext("gmi", &basepath), thread.to_gmi().as_bytes());
            }
            // this is a bit awkward
            self.thread_topics.push(ThreadSummary {
                message: thread.messages[0].clone(),
                reply_count: (thread.messages.len() - 1) as u64,
                last_reply: thread_ids[thread_ids.len() - 1].time,
            });
            for msg in thread.messages {
                let base_path = message_dir.join(&msg.pathescape_msg_id());
                write_if_unchanged(&append_ext("eml", &base_path), &msg.export_eml());
            }
        }
        self.thread_topics.sort_by_key(|t| t.last_reply);
        self.thread_topics.reverse();
        self.write_index();
    }
}

fn main() -> Result<()> {
    let args = arg::Args::from_env();
    let maildir = &args.positional[0];
    let mut config = Config::from_file(&args.config)?;
    // TODO cleanup
    config.include_gemini = args.include_gemini;
    config.out_dir = args.out_dir;
    INSTANCE.set(config).unwrap();

    let mut lists = Lists {
        lists: vec![],
        out_dir: Config::global().out_dir.clone(),
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
        lists.add(list, &dir_name);
    }

    lists.write_lists();
    Ok(())
}

// TODO del this stuff
//
//
//
//
//
//
//
//
//
//
//     // TODO rename
//     pub fn hash(&self) -> String {
//         self.id.replace("/", ";")
//     }
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
