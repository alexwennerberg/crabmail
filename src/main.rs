// this code is not good
// i am not very good at rust
// that is ok though
#![forbid(unsafe_code)]
use anyhow::Result;
use mail_parser::Message;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use models::*;

use config::{Config, INSTANCE};
mod arg;
mod config;
mod maildir;
mod models;
mod templates;
mod threading;
mod util;

use std::ffi::{OsStr, OsString};

const ATOM_ENTRY_LIMIT: usize = 100;

// stole it from the internet
pub fn append_ext(ext: impl AsRef<OsStr>, path: &PathBuf) -> PathBuf {
    let mut os_string: OsString = path.into();
    os_string.push(".");
    os_string.push(ext.as_ref());
    os_string.into()
}

impl Lists {
    fn write_lists(&mut self) {
        std::fs::create_dir_all(&self.out_dir).ok();
        write_if_changed(&self.out_dir.join("style.css"), include_bytes!("style.css"));
        let base_path = self.out_dir.join("index");
        if Config::global().include_html {
            write_if_changed(&base_path.with_extension("html"), self.to_html());
        }
        if Config::global().include_gemini {
            write_if_changed(&base_path.with_extension("gmi"), self.to_gmi());
        }
        for list in &mut self.lists {
            list.persist();
        }
    }
}

/// Writes the given data to the given path iff the data is not
/// identical to the file contents.
///
/// Returns true if it wrote to the file.
fn write_if_changed<T: AsRef<[u8]>>(path: &PathBuf, data: T) -> bool {
    // TODO: use checksum / cache
    if let Ok(d) = std::fs::read(path) {
        if d == data.as_ref() {
            return false;
        }
    }

    std::fs::write(path, data).unwrap();
    true
}

impl List {
    fn persist(&mut self) {
        self.write_threads();
    }
    fn write_index(&self) {
        fn write(elems: Vec<String>, out_dir: &Path, ext: &str) {
            for (n, text) in elems.iter().enumerate() {
                let index = if n == 0 {
                    out_dir.join("index")
                } else {
                    out_dir.join(format!("{}-{}", "index", n + 1))
                };
                write_if_changed(&index.with_extension(ext), text);
            }
        }

        if Config::global().include_gemini {
            write(self.to_gmi(), &self.out_dir, "gmi");
        }
        if Config::global().include_html {
            write(self.to_html(), &self.out_dir, "html");
        }
        write_if_changed(&self.out_dir.join("atom.xml"), self.to_xml());
    }

    // Used with atom
    fn get_recent_messages(&self) -> Vec<StrMessage> {
        let mut out = Vec::new();
        let mut msgs: Vec<&threading::Msg> = self.thread_idx.threads.iter().flatten().collect();
        msgs.sort_by(|a, b| a.time.cmp(&b.time));
        msgs.reverse();
        for m in msgs.iter().take(ATOM_ENTRY_LIMIT) {
            let data = std::fs::read(&m.path).unwrap();
            let msg = StrMessage::new(&Message::parse(&data).unwrap());
            out.push(msg);
        }
        out
    }

    fn write_threads(&mut self) {
        // wonky
        let mut files_written: HashSet<PathBuf> = HashSet::new();
        let thread_dir = self.out_dir.join("threads");
        let message_dir = self.out_dir.join("messages");
        std::fs::create_dir_all(&thread_dir).ok();
        std::fs::create_dir_all(&message_dir).ok();
        for thread_ids in &self.thread_idx.threads {
            // Load thread
            let mut thread = Thread::new(thread_ids, &self.config.name, &self.config.email);
            let basepath = thread_dir.join(&thread.messages[0].pathescape_msg_id());
            // this is a bit awkward
            let summary = ThreadSummary {
                message: thread.messages[0].clone(),
                reply_count: (thread.messages.len() - 1) as u64,
                last_reply: thread_ids[thread_ids.len() - 1].time.clone(),
            };
            for msg in &mut thread.messages {
                msg.set_url(self, &summary); // awkward) // hacky
            }
            self.thread_topics.push(summary);
            if Config::global().include_html {
                let html = append_ext("html", &basepath);
                write_if_changed(&html, thread.to_html());
                files_written.insert(html);
            }
            let xml = append_ext("xml", &basepath);
            write_if_changed(&xml, thread.to_xml());
            files_written.insert(xml);
            if Config::global().include_gemini {
                let gmi = append_ext("gmi", &basepath);
                write_if_changed(&gmi, thread.to_gmi());
                files_written.insert(gmi);
            }

            for msg in thread.messages {
                let eml = append_ext("mbox", &message_dir.join(&msg.pathescape_msg_id()));
                write_if_changed(&eml, &msg.export_mbox());
                files_written.insert(eml);
            }
        }
        self.thread_topics
            .sort_by(|a, b| a.last_reply.cmp(&b.last_reply));
        self.thread_topics.reverse();
        self.recent_messages = self.get_recent_messages();
        // for msg in &mut self.recent_messages {
        // TBD
        // msg.set_url(&self, &summary); // awkward) // hacky
        // }
        // Remove deleted stuff
        for dir in vec![message_dir, thread_dir] {
            for entry in fs::read_dir(&dir).unwrap() {
                match entry {
                    Ok(e) => {
                        if !files_written.contains(&e.path()) {
                            fs::remove_file(&e.path()).ok();
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
        //
        self.write_index();
    }
}

fn main() {
    let args = arg::Args::from_env();
    let maildir = &args.positional[0];
    let mut config = Config::from_file(&args.config).expect("could not read config");
    // Default to both true if both absent
    if !args.include_gemini && !args.include_html {
        config.include_gemini = true;
        config.include_html = true;
    } else {
        config.include_gemini = args.include_gemini;
        config.include_html = args.include_html;
    }
    config.out_dir = args.out_dir;
    INSTANCE.set(config).unwrap();

    // TODO allow one level lower -- one list etc
    let mut lists = Lists {
        lists: vec![],
        out_dir: Config::global().out_dir.clone(),
    };
    for maildir in std::fs::read_dir(maildir)
        .expect("could not read maildir")
        .filter_map(|m| m.ok())
    {
        let dir_name = maildir.file_name().into_string().unwrap(); // TODO no unwrap
        if dir_name.starts_with('.') || ["cur", "new", "tmp"].contains(&dir_name.as_str()) {
            continue;
        }

        let mut list = threading::ThreadIdx::new();
        let mail = maildir::list_all(maildir.path())
            .expect("maildir configured incorrectly")
            .filter_map(Result::ok);

        for entry in mail {
            let path = entry.path().to_path_buf();
            let data = std::fs::read(&path).expect("could not read mail");

            if let Some(mail) = mail_parser::Message::parse(&data) {
                list.add_email(&mail, path);
            } else {
                println!("Could not parse message {:?}", path);
            }
        }

        list.finalize();
        lists.add(list, &dir_name);
    }

    lists.write_lists();
}
