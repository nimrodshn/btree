use crate::error::Error;
use crate::node::{Node, PAGE_SIZE, ROOT_NODE_OFFSET};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;

pub struct Pager<'a> {
    fd: File,
    cache: HashMap<String, &'a Node>,
}

impl<'a> Pager<'a> {
    pub fn new(path: &Path) -> Result<Pager, io::Error> {
        let fd = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(path)?;

        Ok(Pager {
            fd: fd,
            cache: HashMap::<String, &Node>::new(),
        })
    }

    /// find_page looks up a node which holds the value of a certain key.
    /// If no such node exist an error of type KeyNotFound is returned.
    /// 
    /// find_page first tries to find the node in the in-memory cache and if the search misses.
    /// we will look up the corresponding node from file.
    pub fn find_page(&mut self, key: &String) -> Result<&Node, Error> {
        match self.cache.get(key) {
            Some(node) => return Ok(*node),
            None => return self.load_page_from_memory(key),
        };
    }

    pub fn load_page_from_memory(&mut self, key: &String) -> Result<&Node, Error> {
        let mut root_page = vec![0u8; PAGE_SIZE as usize];
        match self.fd.read_exact(&mut root_page) {
            Err(_e) => return Err(Error::UnexpectedError),
            Ok(v) => v,
        };
        let root = Node::page_to_node(ROOT_NODE_OFFSET, &root_page);
        Err(Error::KeyNotFound)
    }
}
