// Extremely minimalist command line interface, inspired by
// [sbase](https://git.suckless.org/sbase/)'s
// [arg.h](https://git.suckless.org/sbase/file/arg.h.html)
//
// I believe this has the same behavior, which is:
// * flags can be grouped (-abc)
// * missing arg -> print usage, exit
// * invalid flag -> print usage, exit
//
// This is, of course, aggressively minimalist, perhaps even too much so.
//
// Copy/paste this code and you have a CLI! No library needed!

use std::env;
use std::path::PathBuf;
use std::process::exit;

fn usage() -> ! {
    let name = env::args().next().unwrap();
    eprintln!(
        "usage: {} [maildir]
FLAGS 
-r  use relative timestamps 

ARGS:
-c  config file (crabmail.conf)
-d  output directory (site)",
        name
    );
    exit(1)
}

pub struct Args {
    pub maildir: String,
    pub config: PathBuf,
    pub out_dir: PathBuf,
    pub flags: String,
}

impl Args {
    pub fn from_env() -> Self {
        // Modify as needed
        let mut flags = String::new();
        let mut maildir: Option<String> = None;
        let mut out_dir = "site".into();
        let mut config = "crabmail.conf".into();

        let mut args = env::args().skip(1);

        // Doesn't support non-UTF-8 paths TODO: solution?
        // See https://github.com/RazrFalcon/pico-args/issues/2
        let parsenext =
            |a: Option<String>| a.and_then(|a| a.parse().ok()).unwrap_or_else(|| usage());

        while let Some(arg) = args.next() {
            let mut chars = arg.chars();
            if chars.next() != Some('-') {
                maildir = Some(arg);
                continue;
            }
            chars.for_each(|m| match m {
                'c' => config = parsenext(args.next()),
                'd' => out_dir = parsenext(args.next()),
                'r' => flags.push(m),
                _ => {
                    usage();
                }
            })
        }
        Self {
            config,
            maildir: match maildir {
                Some(m) => m.into(),
                None => usage(),
            },
            out_dir,
            flags,
        }
    }
}
