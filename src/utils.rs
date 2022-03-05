use std::fs::{read, write};
use std::io::prelude::*;
use std::path::PathBuf;

// TODO: use checksum / cache. bool whether it writes
fn write_if_unchanged(path: PathBuf, data: &[u8]) -> bool {
    if let Ok(d) = read(&path) {
        if &d == data {
            return false;
        }
    } else {
        write(&path, data).unwrap()
    }
    return true;
}
