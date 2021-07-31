use anyhow::Result;
use askama::Template;
use mailparse::{dateparse, parse_headers, parse_mail, MailHeaderMap, ParsedMail};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

mod filters;
mod utils;

const HELP: &str = "\
Usage: crabmail (THIS STRING IS JUNK)

FLAGS:
  -h, --help           Prints this help information and exits.
  -v, --version        Prints the version and exits
  -t, --threads        Group messages into threads

OPTIONS:
  -d, --dir            Directory to save the HTML files in
  -m, --mbox           Mbox file, files, or directories to read in
";

// TODO be more clear about the expected input types
// maildi

#[derive(Debug)]
struct RawEmail {
    date: i64, // unix
    data: Vec<u8>,
}

// TODO refactor
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
    // this function doesnt do what I want
    let in_mboxes = pargs.values_from_os_str(["-m", "--mail"], parse_path)?;
    if in_mboxes.len() == 0 {
        println!("Please provide an input folder");
        std::process::exit(1);
    }

    // Maps thread msg id to all items in the thread
    let mut threads: HashMap<String, Vec<RawEmail>> = HashMap::new();

    for file in std::fs::read_dir(&in_mboxes[0])? {
        // assuming one email per file for now
        let mut buffer = Vec::new();
        let mut f = File::open(&file?.path())?;
        f.read_to_end(&mut buffer)?;
        let (headers, _) = parse_headers(&buffer)?;
        let msg_id = headers.get_first_value("message-id").unwrap(); // TODO error
        let in_reply_to = headers.get_first_value("in-reply-to");
        // Note that date can be forged by the client
        let date = dateparse(
            &headers
                .get_first_value("date")
                .unwrap_or(String::from("-1")),
        )?;

        let message = RawEmail {
            date: date,
            data: buffer,
        };

        // TODO clean message id
        match in_reply_to {
            Some(irt) => {
                if threads.get(&irt).is_none() {
                    threads.insert(irt, vec![message]);
                } else {
                    threads.get_mut(&irt).unwrap().push(message);
                }
            }
            None => {
                threads.insert(msg_id, vec![message]);
            }
        }
    }

    // sort items in each thread by date
    for (key, mut value) in &mut threads {
        value.sort_by(|a, b| a.date.cmp(&b.date));
    }

    // TODO generate thread list sorted by most recent email in thread
    std::fs::create_dir(&out_dir).ok();
    let thread_dir = &out_dir.join("threads");
    std::fs::create_dir(thread_dir).ok();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(out_dir.join("index.html"))?;
    let thread_list = ThreadList {
        thread_ids: threads.keys().collect(),
    };
    file.write(thread_list.render()?.as_bytes());
    // TODO prevent path traversal bug from ./.. in message id
    for (key, value) in threads {
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(thread_dir.join(&key))?;
        let thread = Thread {
            messages: value.iter().map(|m| parse_mail(&m.data).unwrap()).collect(),
        };
        file.write(thread.render()?.as_bytes());
    }
    Ok(())
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

#[derive(Template)]
#[template(path = "thread.html")] // using the template in this path, relative
struct Thread<'a> {
    messages: Vec<ParsedMail<'a>>,
}

#[derive(Template)]
#[template(path = "threadlist.html")] // using the template in this path, relative
struct ThreadList<'a> {
    thread_ids: Vec<&'a String>,
}
