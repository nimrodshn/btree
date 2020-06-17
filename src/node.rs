pub mod file;

/// Node Type
pub enum NodeType {
    Internal = 1,
    Leaf = 2,
}

/// Node represents a node in the BTree occupied by a single page in memory.
pub struct Node {
    pub node_type: NodeType,
    pub offset: u32,
    pub parent_offset: u32,
    pub is_root: bool,
    pub leaf: bool,
}

impl Node {
    pub fn new(
        node_type: NodeType,
        leaf: bool,
        offset: u32,
        parent_offset: u32,
        is_root: bool,
    ) -> Node {
        Node {
            node_type,
            offset,
            parent_offset,
            is_root,
            leaf,
        }
    }
}
