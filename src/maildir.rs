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
use std::error;
use std::fmt;
use std::fs;
use std::io::prelude::*;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug)]
pub enum MailEntryError {
    IOError(std::io::Error),
    DateError(&'static str),
}

impl fmt::Display for MailEntryError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            MailEntryError::IOError(ref err) => write!(f, "IO error: {}", err),
            MailEntryError::DateError(ref msg) => write!(f, "Date error: {}", msg),
        }
    }
}

impl error::Error for MailEntryError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            MailEntryError::IOError(ref err) => Some(err),
            MailEntryError::DateError(_) => None,
        }
    }
}

impl From<std::io::Error> for MailEntryError {
    fn from(err: std::io::Error) -> MailEntryError {
        MailEntryError::IOError(err)
    }
}

impl From<&'static str> for MailEntryError {
    fn from(err: &'static str) -> MailEntryError {
        MailEntryError::DateError(err)
    }
}

enum MailData {
    None,
    #[cfg(not(feature = "mmap"))]
    Bytes(Vec<u8>),
    #[cfg(feature = "mmap")]
    File(memmap::Mmap),
}

impl MailData {
    fn is_none(&self) -> bool {
        match self {
            MailData::None => true,
            _ => false,
        }
    }
}

/// This struct represents a single email message inside
/// the maildir. Creation of the struct does not automatically
/// load the content of the email file into memory - however,
/// that may happen upon calling functions that require parsing
/// the email.
pub struct MailEntry {
    id: String,
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
    path: PathBuf,
    subfolder: Subfolder,
    readdir: Option<fs::ReadDir>,
}

impl MailEntries {
    fn new(path: PathBuf, subfolder: Subfolder) -> MailEntries {
        MailEntries {
            path,
            subfolder,
            readdir: None,
        }
    }
}

impl Iterator for MailEntries {
    type Item = std::io::Result<MailEntry>;

    fn next(&mut self) -> Option<std::io::Result<MailEntry>> {
        if self.readdir.is_none() {
            let mut dir_path = self.path.clone();
            dir_path.push(match self.subfolder {
                Subfolder::New => "new",
                Subfolder::Cur => "cur",
            });
            self.readdir = match fs::read_dir(dir_path) {
                Err(_) => return None,
                Ok(v) => Some(v),
            };
        }

        loop {
            // we need to skip over files starting with a '.'
            let dir_entry = self.readdir.iter_mut().next().unwrap().next();
            let result = dir_entry.map(|e| {
                let entry = e?;
                let filename = String::from(entry.file_name().to_string_lossy().deref());
                if filename.starts_with('.') {
                    return Ok(None);
                }
                let (id, flags) = match self.subfolder {
                    Subfolder::New => (Some(filename.as_str()), Some("")),
                    Subfolder::Cur => {
                        let mut iter = filename.split(":2,");
                        (iter.next(), iter.next())
                    }
                };
                if id.is_none() || flags.is_none() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "Non-maildir file found in maildir",
                    ));
                }
                Ok(Some(MailEntry {
                    id: String::from(id.unwrap()),
                    flags: String::from(flags.unwrap()),
                    path: entry.path(),
                }))
            });
            return match result {
                None => None,
                Some(Err(e)) => Some(Err(e)),
                Some(Ok(None)) => continue,
                Some(Ok(Some(v))) => Some(Ok(v)),
            };
        }
    }
}

#[derive(Debug)]
pub enum MaildirError {
    Io(std::io::Error),
    Utf8(std::str::Utf8Error),
    Time(std::time::SystemTimeError),
}

impl fmt::Display for MaildirError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use MaildirError::*;

        match *self {
            Io(ref e) => write!(f, "IO Error: {}", e),
            Utf8(ref e) => write!(f, "UTF8 Encoding Error: {}", e),
            Time(ref e) => write!(f, "Time Error: {}", e),
        }
    }
}

impl error::Error for MaildirError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        use MaildirError::*;

        match *self {
            Io(ref e) => Some(e),
            Utf8(ref e) => Some(e),
            Time(ref e) => Some(e),
        }
    }
}

impl From<std::io::Error> for MaildirError {
    fn from(e: std::io::Error) -> MaildirError {
        MaildirError::Io(e)
    }
}
impl From<std::str::Utf8Error> for MaildirError {
    fn from(e: std::str::Utf8Error) -> MaildirError {
        MaildirError::Utf8(e)
    }
}
impl From<std::time::SystemTimeError> for MaildirError {
    fn from(e: std::time::SystemTimeError) -> MaildirError {
        MaildirError::Time(e)
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
    pub fn list_new(&self) -> MailEntries {
        MailEntries::new(self.path.clone(), Subfolder::New)
    }

    /// Returns an iterator over the messages inside the `cur`
    /// maildir folder. The order of messages in the iterator
    /// is not specified, and is not guaranteed to be stable
    /// over multiple invocations of this method.
    pub fn list_cur(&self) -> MailEntries {
        MailEntries::new(self.path.clone(), Subfolder::Cur)
    }
}

impl From<PathBuf> for Maildir {
    fn from(p: PathBuf) -> Maildir {
        Maildir { path: p }
    }
}

impl From<String> for Maildir {
    fn from(s: String) -> Maildir {
        Maildir::from(PathBuf::from(s))
    }
}

impl<'a> From<&'a str> for Maildir {
    fn from(s: &str) -> Maildir {
        Maildir::from(PathBuf::from(s))
    }
}
