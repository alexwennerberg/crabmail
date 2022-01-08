use once_cell::sync::OnceCell;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// Config file structure is very simple:
// key=value\n
#[derive(Debug)]
pub struct Config {
    pub email_fmt: String,
    pub base_url: String,
    pub out_dir: PathBuf,
    pub relative_times: bool,
    pub include_raw: bool,
}

// TODO list-specific config

pub static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("Config is not initialized")
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, std::io::Error> {
        let file = File::open(path)?;
        let mut email_fmt = "lists+%s@example.com".to_string();
        let mut base_url = "https://example.com".to_string();

        for l in io::BufReader::new(file).lines() {
            let line = l?;
            if line.len() == 0 {
                continue;
            }
            if let Some(i) = line.find('=') {
                let key = &line[..i];
                let value = &line[i + 1..];
                match key {
                    "email_fmt" => email_fmt = value.to_string(),
                    "base_url" => base_url = value.to_string(),
                    _ => {}
                }
            } else {
                // panic!("Invalid config")
            }
        }
        Ok(Config {
            email_fmt,
            base_url,
            out_dir: PathBuf::from(""),
            relative_times: false,
            include_raw: false,
        })
    }
}
