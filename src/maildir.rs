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
// TODO cleanup
use std::path::PathBuf;

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

enum Subfolder {
    New,
    Cur,
}

/// An iterator over the email messages in a particular
/// maildir subfolder (either `cur` or `new`). This iterator
/// produces a `std::io::Result<MailEntry>`, which can be an
/// `Err` if an error was encountered while trying to read
/// file system properties on a particular entry, or if an
/// invalid file was found in the maildir. Files starting with
/// a dot (.) character in the maildir folder are ignored.
pub struct MailEntries {
    iter: Box<dyn Iterator<Item = std::io::Result<MailEntry>>>,
}

impl MailEntries {
    /// Generates a new MailEntries.
    /// May return an Err if the given path or subfolder are not readable.
    fn new(path: PathBuf, subfolder: Subfolder) -> std::io::Result<MailEntries> {
        let readdir = std::fs::read_dir(path.join(match subfolder {
            Subfolder::New => "new",
            Subfolder::Cur => "cur",
        }))?;

        let iter = readdir
            .map(|maybe_entry| {
                maybe_entry.map(|entry| {
                    let filename = String::from(entry.file_name().to_string_lossy());
                    (filename, entry)
                })
            })
            .filter(|maybe_entry| {
                if let Ok((filename, _)) = maybe_entry {
                    filename.starts_with('.')
                } else {
                    // always keep errors
                    true
                }
            })
            .map(move |maybe_entry| {
                let (filename, entry) = maybe_entry?;
                match subfolder {
                    Subfolder::New => Ok(MailEntry {
                        id: filename,
                        flags: String::new(),
                        path: entry.path(),
                    }),
                    Subfolder::Cur => filename
                        .split_once(":2,")
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
                        }),
                }
            });

        Ok(MailEntries {
            iter: Box::new(iter),
        })
    }
}

impl Iterator for MailEntries {
    type Item = std::io::Result<MailEntry>;

    fn next(&mut self) -> Option<std::io::Result<MailEntry>> {
        self.iter.next()
    }
}

/// The main entry point for this library. This struct can be
/// instantiated from a path using the `from` implementations.
/// The path passed in to the `from` should be the root of the
/// maildir (the folder containing `cur`, `new`, and `tmp`).
pub struct Maildir {
    path: PathBuf,
}

impl Maildir {
    /// Returns an iterator over the messages inside the `new`
    /// maildir folder. The order of messages in the iterator
    /// is not specified, and is not guaranteed to be stable
    /// over multiple invocations of this method.
    pub fn list_new(&self) -> std::io::Result<MailEntries> {
        MailEntries::new(self.path.clone(), Subfolder::New)
    }

    /// Returns an iterator over the messages inside the `cur`
    /// maildir folder. The order of messages in the iterator
    /// is not specified, and is not guaranteed to be stable
    /// over multiple invocations of this method.
    pub fn list_cur(&self) -> std::io::Result<MailEntries> {
        MailEntries::new(self.path.clone(), Subfolder::Cur)
    }

    pub fn list_all(&self) -> std::io::Result<std::iter::Chain<MailEntries, MailEntries>> {
        self.list_cur()
            .and_then(|cur| Ok(cur.chain(self.list_new()?)))
    }
}

impl From<PathBuf> for Maildir {
    fn from(path: PathBuf) -> Maildir {
        Maildir { path }
    }
}
