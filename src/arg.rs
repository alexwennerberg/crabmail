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
//

use std::env;
use std::ffi::OsString;

use std::path::PathBuf;
use std::process::exit;
use std::str::FromStr;

fn usage() -> ! {
    let name = env::args().next().unwrap();
    eprintln!(
        "usage: {} [-rR] [-c CONFIG] [-d OUT_DIR] MAIDLIR

MAILDIR A directory containing the maildirs of lists you want to parse

FLAGS (default -gh if none set)
-g  include gemini output
-h  include HTML output

ARGS:
-c  config file (crabmail.conf)
-d  output directory (site)",
        name
    );
    exit(1)
}

#[derive(Default)]
pub struct Args {
    pub config: PathBuf,
    pub out_dir: PathBuf,
    pub positional: Vec<OsString>,
    pub a: i32, // placeholder
    pub include_gemini: bool,
    pub include_html: bool,
    pub no_index: bool,
}

impl Args {
    pub fn default() -> Self {
        Args {
            out_dir: "site".into(),
            config: "crabmail.conf".into(),
            ..Default::default()
        }
    }

    pub fn from_env() -> Self {
        let mut out = Self::default();
        let mut args = env::args_os().skip(1);
        while let Some(arg) = args.next() {
            let s = arg.to_string_lossy();
            let mut ch_iter = s.chars();
            if ch_iter.next() != Some('-') {
                out.positional.push(arg);
                continue;
            }
            ch_iter.for_each(|m| match m {
                // Edit these lines //
                'c' => out.config = parse_os_arg(args.next()),
                'd' => out.out_dir = parse_os_arg(args.next()),
                'g' => out.include_gemini = true,
                'h' => out.include_html = true,
                // Stop editing //
                _ => {
                    usage();
                }
            })
        }
        // other validation
        if out.positional.len() < 1 {
            usage()
        }
        out
    }
}

#[allow(dead_code)]
fn parse_arg<T: FromStr>(a: Option<OsString>) -> T {
    a.and_then(|a| a.into_string().ok())
        .and_then(|a| a.parse().ok())
        .unwrap_or_else(|| usage())
}

fn parse_os_arg<T: From<OsString>>(a: Option<OsString>) -> T {
    match a {
        Some(s) => T::from(s),
        None => usage(),
    }
}
