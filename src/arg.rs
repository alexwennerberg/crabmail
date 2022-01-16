// This file is licensed under the terms of 0BSD:
//
// Permission to use, copy, modify, and/or distribute this software for any purpose with or without
// fee is hereby granted.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES WITH REGARD TO THIS
// SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE
// AUTHOR BE LIABLE FOR ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION OF CONTRACT,
// NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN CONNECTION WITH THE USE OR PERFORMANCE
// OF THIS SOFTWARE.

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
        "usage: {} [-rR] [-c CONFIG] [-d OUT_DIR] maildir
FLAGS: 
-r  use relative timestamps 
-R  include raw emails [ALPHA]

ARGS:
-c  config file (crabmail.conf)
-d  output directory (site)",
        name
    );
    exit(1)
}

#[derive(Default)]
pub struct Args {
    pub maildir: String,
    pub config: PathBuf,
    pub out_dir: PathBuf,
    pub flags: String,
}

impl Args {
    pub fn from_env() -> Self {
        // Modify as neede
        let mut out = Args {
            out_dir: "site".into(),
            config: "crabmail.conf".into(),
            ..Default::default()
        };

        let mut args = env::args().skip(1);

        let mut maildir = None;
        // Doesn't support non-UTF-8 paths TODO: solution?
        // See https://github.com/RazrFalcon/pico-args/issues/2
        let parsenext =
            |a: Option<String>| a.and_then(|a| a.parse().ok()).unwrap_or_else(|| usage());

        while let Some(arg) = args.next() {
            let mut chars = arg.chars();
            // Positional args
            if chars.next() != Some('-') {
                maildir = Some(arg);
                continue;
            }
            chars.for_each(|m| match m {
                'c' => out.config = parsenext(args.next()),
                'd' => out.out_dir = parsenext(args.next()),
                'r' | 'R' => out.flags.push(m),
                _ => {
                    usage();
                }
            })
        }
        out.maildir = match maildir {
            Some(m) => m.into(),
            None => usage(),
        };
        out
    }
}
