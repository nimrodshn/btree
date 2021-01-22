use crate::error::Error;
use crate::page::Page;
use crate::page_layout::PAGE_SIZE;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

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

    pub fn get_page(&mut self, offset: usize) -> Result<Page, Error> {
        let mut page: [u8; PAGE_SIZE] = [0x00; PAGE_SIZE];
        self.file.seek(SeekFrom::Start(offset as u64))?;
        self.file.read_exact(&mut page)?;
        Ok(Page::new(page))
    }

    pub fn write_page(&mut self, page: &Page, offset: &usize) -> Result<(), Error> {
        self.file.seek(SeekFrom::Start(*offset as u64))?;
        self.file.write_all(&page.get_data())?;
        Ok(())
    }
}
