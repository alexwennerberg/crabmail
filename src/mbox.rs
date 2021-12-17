// basically just rewrite of https://github.com/emersion/go-mbox/blob/master/reader.go
// so-called "mboxo" mbox format https://www.loc.gov/preservation/digital/formats/fdd/fdd000384.shtml
// (which has minor issues, described therein)
use anyhow::Result;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

pub struct MboxReader<T>
where
    T: BufRead,
{
    in_record: bool,
    done: bool,
    reader: T,
}

// Impl iterator
// Next()
impl<T: BufRead> MboxReader<T> {
    pub fn from_reader(reader: T) -> MboxReader<T> {
        return MboxReader {
            in_record: false,
            done: false,
            reader,
        };
    }
}

// ugly im bad at rust
pub fn from_file(p: &Path) -> Result<MboxReader<BufReader<File>>> {
    let f = File::open(p)?;
    let mut reader = BufReader::new(f);
    let mut mboxr = MboxReader::from_reader(reader);
    Ok(mboxr)
}
// not the prettiest but it works
impl<T: BufRead> Iterator for MboxReader<T> {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn next(&mut self) -> Option<Self::Item> {
        // mbox is ASCII, so don't use String
        if self.done {
            return None;
        }
        let mut record = Vec::new();
        let mut current_line = Vec::new();
        let mut started = true;
        let mut res = 1;
        while res != 0 {
            current_line = vec![];
            res = match self.reader.read_until(b'\n', &mut current_line) {
                Ok(l) => l,
                Err(err) => return Some(Err(err)),
            };

            if current_line.starts_with(b"From ") {
                if !self.in_record {
                    self.in_record = true;
                    continue;
                } else {
                    break;
                }
            }
            // MBOXO escaping
            if current_line.starts_with(b">From") {
                record.extend_from_slice(&current_line[1..]);
            } else {
                record.extend_from_slice(&current_line);
            }
        }
        if res == 0 {
            self.done = true;
        }
        if record.ends_with(b"\r\n\r\n") {
            // Remove double CRLF
            record.pop();
            record.pop();
        }
        Some(Ok(record))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    #[test]
    fn test_simple() {
        let data =
            "From something\r\nmyemail\r\nmessage-id: 123\r\n\r\nFrom Another\r\n>From escape\r\n\r\n";
        let mut reader = BufReader::new(data.as_bytes());
        let mut mboxr = MboxReader::from_reader(reader);
        assert_eq!(
            str::from_utf8(&mboxr.next().unwrap().unwrap()).unwrap(),
            "myemail\r\nmessage-id: 123\r\n"
        );
        assert_eq!(
            str::from_utf8(&mboxr.next().unwrap().unwrap()).unwrap(),
            "From escape\r\n"
        );
        assert!(mboxr.next().is_none());
    }
}
