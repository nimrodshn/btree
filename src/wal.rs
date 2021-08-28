use std::path::{PathBuf};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{Read, Seek, SeekFrom};
use crate::error::Error;

struct WAL {
    file: File,
    curser: usize,
}

impl WAL {
    fn new(parent_directoy: PathBuf) -> Result<Self, Error> {
        let fd = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(true)
            .open(parent_directoy.join("wal"))?;

        Ok(Self {
            file: fd,
            curser: 0,
        })
    }
}