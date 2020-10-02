use crate::node::Node;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::path::Path;

// Root Node offset is always at zero.
pub const ROOT_NODE_OFFSET: usize = 0;

pub struct Pager {
    fd: File,
    cache: HashMap<String, Box<Node>>,
}

impl Pager {
    pub fn new(path: &Path) -> Result<Pager, io::Error> {
        let fd = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;

        Ok(Pager {
            fd: fd,
            cache: HashMap::<String, Box<Node>>::new(),
        })
    }
}
