use anyhow::Result;
use mailparse::{parse_mail, MailHeaderMap, ParsedMail};
use std::fs::{File, OpenOptions};
use std::io::prelude::*;
use std::path::Path;

mod utils;

const HELP: &str = "\
Usage: crabmail

FLAGS:
  -h, --help           Prints this help information and exits.
  -v, --version        Prints the version and exits
  -t, --threads        Group messages into threads

OPTIONS:
  -d, --dir            Directory to save the HTML files in
  -m, --mbox           Mbox file, files, or directories to read in
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
    // Create if does not exist
    let in_mboxes = pargs.values_from_os_str(["-m", "--mbox"], parse_path)?;
    if in_mboxes.len() == 0 {
        println!("Please provide one or more input files with the -m flag");
        std::process::exit(1);
    }
    for file in in_mboxes {
        // assuming one email per file for now
        let mut buffer = Vec::new();
        let mut f = File::open(&file)?;
        f.read_to_end(&mut buffer)?;
        let email = parse_mail(&buffer)?;
        println!("{}", email_to_html(email));
    }
    if pargs.contains(["-t", "--threads"]) {
        // TODO
    }

    std::fs::create_dir(&out_dir).ok();
    write_index(&out_dir)?;
    Ok(())
}

fn parse_path(s: &std::ffi::OsStr) -> Result<std::path::PathBuf, &'static str> {
    Ok(s.into())
}

fn email_to_html(email: ParsedMail) -> String {
    // TODO use format strings here i think
    return format!(
        r#"
<b>From<b>: {from}<br>
<b>Subject</b>: {subject}
<div id="body"> {body} </div>
    "#,
        from = &email
            .headers
            .get_first_value("From")
            .unwrap_or("".to_string()),
        subject = &email
            .headers
            .get_first_value("Subject")
            .unwrap_or("".to_string()),
        body = &email.get_body().unwrap()
    );
}

// TODO set lang, title, etc
const HEADER: &[u8] = br#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<link rel="stylesheet" type="text/css" href="/style.css">
</head>
<body>
<main>
"#;

const FOOTER: &[u8] = br#"
</main>
</body>
</html>
"#;

// TODO write wrapper
fn write_index(out_dir: &Path) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(out_dir.join("index.html"))?;
    file.write_all(HEADER)?;
    file.write_all(b"<h1>Hello world</h1>")?;
    file.write_all(FOOTER)?;
    Ok(())
}
