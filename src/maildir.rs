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

// Vendoring https://github.com/staktrace/maildir
use std::{
    io::Result,
    fs::DirEntry,
    path::PathBuf,
};

/// This struct represents a single email message inside
/// the maildir. Creation of the struct does not automatically
/// load the content of the email file into memory - however,
/// that may happen upon calling functions that require parsing
/// the email.
pub struct MailEntry {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    flags: String,
    path: PathBuf,
}

impl MailEntry {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

/// Generates a new MailEntries.
/// May return an Err if the given path or subfolder are not readable.
pub fn list_all(path: PathBuf) -> Result<impl Iterator<Item = Result<MailEntry>>> {
    type MapResult = Result<(String, DirEntry)>;

    fn file_iter(path: PathBuf) -> Result<impl Iterator<Item = MapResult>> {
        Ok(std::fs::read_dir(path.join("new"))?
            .map(|maybe_entry| maybe_entry.map(|entry| {
                let filename = String::from(entry.file_name().to_string_lossy());
                (filename, entry)
            }))
            .filter(|maybe_entry| {
                // always keep errors, otherwise only keep if they dont start with dots
                maybe_entry.as_ref().map_or(true, |(filename, _)| !filename.starts_with('.'))
            }))
    }

    fn parse_new(maybe_entry: MapResult) -> Result<MailEntry> {
        maybe_entry.map(|(filename, entry)| MailEntry {
            id: filename,
            flags: String::new(),
            path: entry.path(),
        })
    }

    fn parse_cur(maybe_entry: MapResult) -> Result<MailEntry> {
        maybe_entry.and_then(|(filename, entry)| {
            filename.split_once(":2,")
                .map(|(id, flags)| MailEntry {
                    id: id.to_string(),
                    flags: flags.to_string(),
                    path: entry.path(),
                })
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Non-maildir file found in maildir",
                    )
                })
        })
    }

    let iter_new = file_iter(path.join("new"))?.map(parse_new);
    let iter_cur = file_iter(path.join("cur"))?.map(parse_cur);

    Ok(iter_new.chain(iter_cur))
}
