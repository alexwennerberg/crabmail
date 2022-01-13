use once_cell::sync::OnceCell;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::{Path, PathBuf};

// Ini-like configuration, with sections.
// Global config first, then config for each subsection
// key=value
//
// [section]
// key2=value2
#[derive(Default, Debug)]
pub struct Config {
    pub email_fmt: String,
    pub base_url: String,
    pub out_dir: PathBuf,     // TODO rem
    pub relative_times: bool, // TODO rem
    pub include_raw: bool,    // TODO rem
    pub now: String,          // TODO rem
    pub subsections: Vec<Subsection>,
}

impl Config {
    pub fn match_kv(&mut self, key: &str, value: &str) {
        match key {
            "email_fmt" => self.email_fmt = value.to_string(),
            "base_url" => self.base_url = value.to_string(),
            _ => {}
        }
    }
}

#[derive(Default, Debug)]
pub struct Subsection {
    pub name: String,  // something
    pub title: String, // something mailing list
    pub email: String,
    pub description: String,
}

impl Subsection {
    fn match_kv(&mut self, key: &str, value: &str) {
        match key {
            "title" => self.title = value.to_string(),
            "email" => self.email = value.to_string(),
            "description" => self.description = value.to_string(),
            _ => {}
        }
    }
}

pub static INSTANCE: OnceCell<Config> = OnceCell::new();

impl Config {
    pub fn global() -> &'static Config {
        INSTANCE.get().expect("Config is not initialized")
    }

    pub fn default_subsection(&self, name: &str) -> Subsection {
        Subsection {
            name: name.to_owned(),
            title: format!("{} mailing list", name),
            email: self.email_fmt.replace("%s", name),
            description: String::new(),
        }
    }
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Config, std::io::Error> {
        let file = File::open(path)?;
        // let sub_sections = vec![];
        let mut conf = Config::default();
        let mut current_section = None;

        for l in io::BufReader::new(file).lines() {
            let line = l?;
            if line.starts_with("[") && line.ends_with("]") {
                let name = &line[1..line.len() - 1];
                // Defaults from global config
                if current_section.is_some() {
                    conf.subsections.push(current_section.unwrap());
                }
                current_section = Some(conf.default_subsection(name))
            }
            if line.len() == 0 {
                continue;
            }
            if let Some(i) = line.find('=') {
                let key = &line[..i];
                let value = &line[i + 1..];
                if let Some(ref mut s) = current_section {
                    s.match_kv(key, value);
                }
                conf.match_kv(key, value);
            } else {
                // panic!("Invalid config")
            }
        }
        if current_section.is_some() {
            conf.subsections.push(current_section.unwrap());
        }
        conf.now = crate::time::current_time_rfc3339(); // TODO remove
        Ok(conf)
    }
}
