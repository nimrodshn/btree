use crate::error::Error;
use crate::node::{Node, NodeType};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io;
use std::io::prelude::*;
use std::path::Path;

/// A single page size.
/// Each page represents a node in the BTree.
pub const PAGE_SIZE: usize = 4096;

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

    /// find_page looks up a node which holds the value of a certain key.
    /// If no such node exist an error of type KeyNotFound is returned.
    ///
    /// find_page first tries to find the node in the in-memory cache and if the search misses.
    /// we will look up the corresponding node from file.
    pub fn find_node(&mut self, key: String) -> Result<Node, Error> {
        match self.cache.get(&key) {
            Some(node) => return Ok(*node.clone()),
            None => return self.load_node_from_memory(key),
        };
    }

    pub fn load_node_from_memory(&mut self, key: String) -> Result<Node, Error> {
        let mut root_page = vec![0u8; PAGE_SIZE as usize];
        match self.fd.read_exact(&mut root_page) {
            Err(_e) => return Err(Error::UnexpectedError),
            Ok(v) => v,
        };
        let root = Node::page_to_node(ROOT_NODE_OFFSET, &root_page)?;
        // In case the root is also a leaf.
        if root.node_type == NodeType::Leaf {
            let kv_pairs = root.get_key_value_pairs(&root_page)?;
            for kv in kv_pairs.iter() {
                if kv.key == key {
                    return Ok(root);
                }
            }
            // Store the root in the in-memory cache.
            self.cache.insert(key, Box::new(root));
        } else {
            // Do something here.
        }

        Err(Error::KeyNotFound)
    }
}
