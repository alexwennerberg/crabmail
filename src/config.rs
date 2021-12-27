use once_cell::sync::OnceCell;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// Config file structure is very simple:
// key=value\n
#[derive(Debug)]
pub struct Config {
    pub list_name: String,
    pub list_email: String,
    pub url: String,
    pub homepage: String,
    // mimes that will be preserved as raw attachment files
    // wildcards allowed as *
    // WIP
    pub ok_attachments: Vec<String>,
    pub out_dir: PathBuf,
    pub relative_times: bool,
}

pub static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("Config is not initialized")
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, std::io::Error> {
        let file = File::open(path)?;
        let mut list_name = "Crabmail Mailing List".to_string();
        let mut list_email = "setme@foo.local".to_string();
        let mut url = "flounder.online".to_string();
        let mut ok_attachments = vec!["text/python".to_string()];
        let mut homepage = String::new();

        for l in io::BufReader::new(file).lines() {
            let line = l?;
            if line.len() == 0 {
                continue;
            }
            if let Some(i) = line.find('=') {
                let key = &line[..i];
                let value = &line[i + 1..];
                match key {
                    "list_name" => list_name = value.to_string(),
                    "list_email" => list_email = value.to_string(),
                    "url" => url = value.to_string(),
                    "homepage" => homepage = value.to_string(),
                    "ok_attachments" => {
                        ok_attachments = value.split(",").map(|s| s.to_owned()).collect()
                    }
                    _ => {}
                }
            } else {
                // Replace with whatever you want to do on malformed config lines
                panic!("Invalid config")
            }
        }
        Ok(Config {
            list_name,
            list_email,
            url,
            homepage,
            out_dir: PathBuf::from(""),
            ok_attachments,
            relative_times: false,
        })
    }
}
