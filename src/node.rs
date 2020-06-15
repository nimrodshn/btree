use std::error::Error;
use std::option::Option;

pub mod file;

pub struct KeyValuePair{
    key: String,
    value: String,
}

/// Node represents a node in the BTree occupied by a single page in memory.
pub struct Node
{
    pub offset: u32,
    pub parent_offset: u32,
    pub leaf: bool,
}

impl Node 
{
    pub fn new(
        leaf: bool,
        offset: u32,
        parent_offset: u32,
    ) -> Node {
        Node {
            leaf,
            offset,
            parent_offset,
        }
    }
}
