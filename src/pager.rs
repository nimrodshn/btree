use crate::error::Error;
use crate::page::PAGE_SIZE;
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

// Root Node offset is always at zero.
pub const ROOT_NODE_OFFSET: usize = 0;

pub struct Pager {
    file: File,
}

impl Pager {
    pub fn new(path: &Path) -> Result<Pager, Error> {
        let fd = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;

        Ok(Pager { file: fd })
    }

    pub fn get_page(&mut self, offset: usize) -> Result<[u8; PAGE_SIZE], Error> {
        let mut page: [u8; PAGE_SIZE] = [0x00; PAGE_SIZE];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read_exact(&mut page)?;
        Ok(page)
    }
}
