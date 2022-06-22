// this code is not good
// i am not very good at rust
// that is ok though
#[forbid(unsafe_code)]
use anyhow::Result;
use mail_parser::Message;
use maildir::Maildir;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

use models::*;

use config::{Config, INSTANCE};
mod arg;
mod config;
mod maildir;
mod models;
mod templates;
mod threading;
mod time;
mod util;

const ATOM_ENTRY_LIMIT: usize = 100;

impl Lists {
    fn write_lists(&mut self) {
        std::fs::create_dir_all(&self.out_dir).ok();
        let css = include_bytes!("style.css");
        write_if_changed(&self.out_dir.join("style.css"), css);
        let base_path = self.out_dir.join("index");
        if Config::global().include_html {
            write_if_changed(&base_path.with_extension("html"), self.to_html().as_bytes());
        }
        if Config::global().include_gemini {
            write_if_changed(&base_path.with_extension("gmi"), self.to_gmi().as_bytes());
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
fn write_if_changed(path: &PathBuf, data: &[u8]) -> bool {
    // TODO: use checksum / cache
    if let Ok(d) = std::fs::read(path) {
        if &d == data {
            return false;
        }
    }
    std::fs::write(path, data).unwrap();
    return true;
}

impl List {
    fn persist(&mut self) {
        self.write_threads();
    }
    fn write_index(&self) {
        // TODO fix lazy copy paste
        if Config::global().include_gemini {
            for (n, gmi) in self.to_gmi().iter().enumerate() {
                let index;
                if n == 0 {
                    index = self.out_dir.join("index");
                } else {
                    index = self.out_dir.join(format!("{}-{}", "index", n + 1));
                }
                write_if_changed(&index.with_extension("gmi"), gmi.as_bytes());
            }
        }
        if Config::global().include_html {
            for (n, html) in self.to_html().iter().enumerate() {
                let index;
                if n == 0 {
                    index = self.out_dir.join("index");
                } else {
                    index = self.out_dir.join(format!("{}-{}", "index", n + 1));
                }
                write_if_changed(&index.with_extension("html"), html.as_bytes());
            }
        }
        write_if_changed(&self.out_dir.join("atom.xml"), self.to_xml().as_bytes());
    }

    // Used with atom
    fn get_recent_messages(&self) -> Vec<StrMessage> {
        let mut out = Vec::new();
        let mut msgs: Vec<&threading::Msg> = self.thread_idx.threads.iter().flatten().collect();
        msgs.sort_by_key(|x| x.time);
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
                last_reply: thread_ids[thread_ids.len() - 1].time,
            };
            for msg in &mut thread.messages {
                msg.set_url(&self, &summary); // awkward) // hacky
            }
            self.thread_topics.push(summary);
            if Config::global().include_html {
                let html = basepath.with_extension("html");
                write_if_changed(&html, thread.to_html().as_bytes());
                files_written.insert(html);
            }
            let xml = basepath.with_extension("xml");
            write_if_changed(&xml, thread.to_xml().as_bytes());
            files_written.insert(xml);
            if Config::global().include_gemini {
                let gmi = basepath.with_extension("gmi");
                write_if_changed(&gmi, thread.to_gmi().as_bytes());
                files_written.insert(gmi);
            }

            for msg in thread.messages {
                let eml = message_dir
                    .join(&msg.pathescape_msg_id())
                    .with_extension("mbox");
                write_if_changed(&eml, &msg.export_mbox());
                files_written.insert(eml);
            }
        }
        self.thread_topics.sort_by_key(|t| t.last_reply);
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

fn main() -> Result<()> {
    let args = arg::Args::from_env();
    let maildir = &args.positional[0];
    let mut config = Config::from_file(&args.config)?;
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
            let msg = match mail_parser::Message::parse(&data) {
                Some(e) => e,
                None => {
                    println!("Could not parse message {:?}", f.path());
                    continue;
                }
            };
            list.add_email(&msg, f.path().to_path_buf());
        }
        list.finalize();
        lists.add(list, &dir_name);
    }

    lists.write_lists();
    Ok(())
}
