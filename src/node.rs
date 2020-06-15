pub mod file;

/// Node represents a node in the BTree occupied by a single page in memory.
pub struct Node {
    pub offset: u32,
    pub parent_offset: u32,
    pub is_root: bool,
    pub leaf: bool,
}

impl Node {
    pub fn new(leaf: bool, offset: u32, parent_offset: u32, is_root: bool) -> Node {
        Node {
            offset,
            parent_offset,
            is_root,
            leaf,
        }
    }
}
