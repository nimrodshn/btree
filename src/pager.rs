use crate::node::Node;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

pub struct Pager {
    fd: File,
    cache: HashMap<String, Node>,
}

impl Pager {
    pub fn new(path: &Path) -> Result<Pager, io::Error> {
        let fd = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;

        Ok(Pager {
            fd,
            cache: HashMap::<String, Node>::new(),
        })
    }
}
