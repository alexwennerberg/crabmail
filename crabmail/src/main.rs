use anyhow::{anyhow, Context, Result};
use askama::Template;
use mailparse::{dateparse, parse_headers, parse_mail, MailHeaderMap, ParsedMail};
use mbox_reader::MboxFile;
use std::collections::HashMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use urlencoding::encode;

mod filters;
mod utils;

const HELP: &str = "\
Usage: crabmail 

-m --mbox input mbox file
";

// TODO be more clear about the expected input types
// maildi

// Not a "raw email" struct, but an email object that can be represented by
// crabmail.
#[derive(Debug, Clone)]
struct Email {
    // TODO allocs
    id: String,
    from: String,
    subject: String,
    in_reply_to: Option<String>,
    date: i64, // unix epoch. received date
    body: String,
    // raw_email: String,
}

fn local_parse_email(data: &[u8]) -> Result<Email> {
    let parsed_mail = parse_mail(data)?;
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
            .context("No date header")?,
    )?;
    let from = headers.get_first_value("from").context("No from header")?;
    let body = "lorem ipsum".to_owned();
    return Ok(Email {
        id,
        in_reply_to,
        from,
        subject,
        date,
        body,
    });
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
    let in_mbox = pargs.value_from_os_str(["-m", "--mbox"], parse_path)?;

    let mbox = MboxFile::from_file(&in_mbox)?;

    let mut thread_index: HashMap<String, Vec<String>> = HashMap::new();

    let mut email_index: HashMap<String, Email> = HashMap::new();
    for entry in mbox.iter() {
        let buffer = entry.message().unwrap();
        let email = local_parse_email(buffer)?;
        // TODO fix borrow checker
        if let Some(reply) = email.in_reply_to.clone() {
            match thread_index.get(&reply) {
                Some(e) => {
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

    let mut thread_roots: Vec<&Email> = email_index
        .iter()
        .filter_map(|(k, v)| {
            if v.in_reply_to.is_none() {
                return Some(v);
            }
            return None;
        })
        .collect();
    thread_roots.sort_by_key(|a| a.date);
    thread_roots.reverse();
    std::fs::create_dir(&out_dir).ok();
    let thread_dir = &out_dir.join("threads");
    std::fs::create_dir(thread_dir).ok();
    for root in thread_roots.iter() {
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

        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(thread_dir.join(format!("{}", root.date)))?;
        file.write(Thread { root, messages }.render()?.as_bytes())
            .ok();
    }

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(out_dir.join("index.html"))?;
    file.write(
        ThreadList {
            messages: thread_roots,
        }
        .render()?
        .as_bytes(),
    )
    .ok();

    Ok(())
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

#[derive(Template)]
#[template(path = "thread.html")]
struct Thread<'a> {
    messages: Vec<&'a Email>,
    root: &'a Email,
}

#[derive(Template)]
#[template(path = "threadlist.html")]
struct ThreadList<'a> {
    // message root
    messages: Vec<&'a Email>,
}
