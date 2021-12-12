use anyhow::{anyhow, Context, Result};
use askama::Template;
use mailparse::{dateparse, parse_headers, parse_mail, MailHeaderMap, ParsedMail};
use mbox_reader::MboxFile;
use std::collections::HashMap;
use std::fmt;
use std::fs::{File, OpenOptions};
use std::io::prelude::*;

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
struct Email {
    // TODO allocs
    id: String,
    from: String,
    subject: String,
    in_reply_to: Option<String>,
    date: i64, // unix epoch
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
        return Err(anyhow!("bad message ID"));
    }
    // Assume 1 in-reply-to header. a reasonable assumption
    let in_reply_to = headers.get_first_value("in-reply-to");
    let subject = headers
        .get_first_value("subject")
        .unwrap_or("(no subject)".to_owned());
    let date = dateparse(&headers.get_first_value("date").context("No date header")?)?;
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

    let mut mail_index: HashMap<String, Email> = HashMap::new();
    let mut reply_index: HashMap<String, String> = HashMap::new();

    for entry in mbox.iter() {
        let buffer = entry.message().unwrap();
        // unwrap or warn
        let email = local_parse_email(buffer)?;
    }
    Ok(())
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

#[derive(Template)]
#[template(path = "thread.html")]
struct Thread {
    messages: Vec<Email>,
}

#[derive(Template)]
#[template(path = "threadlist.html")]
struct ThreadList {
    // message root
    messages: Vec<Email>,
}
