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
mod util;

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
            let thread = Thread::new(thread_ids, &self.config.name);
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
