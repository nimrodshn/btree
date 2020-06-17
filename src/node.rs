use crate::error::Error;
use std::mem::size_of;

/// A single page size.
/// Each page represents a node in the BTree.
pub const PAGE_SIZE: usize = 4096;

// Root Node offset is always at zero.
pub const ROOT_NODE_OFFSET: usize = 0;

/// Common Node header layout.
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = 0;
const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 1;
const PARENT_POINTER_OFFSET: usize = 2;
const PARENT_POINTER_SIZE: usize = size_of::<usize>();
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;
/// Leaf node header layout
const LEAF_NODE_NUM_CELLS_SIZE: usize = 4;
const LEAF_NODE_NUM_CELLS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_CELLS_SIZE;

/// Leaf body layout.
const LEAF_NODE_KEY_SIZE: usize = size_of::<usize>() as usize;
const LEAF_NODE_KEY_OFFSET: usize = 0;
const LEAF_NODE_VALUE_SIZE: usize = size_of::<usize>() as usize;
const LEAF_NODE_VALUE_OFFSET: usize = LEAF_NODE_KEY_OFFSET + LEAF_NODE_KEY_SIZE;
const LEAF_NODE_KEY_VALUE_PAIR_SIZE: usize = LEAF_NODE_KEY_SIZE + LEAF_NODE_VALUE_SIZE;
const LEAF_NODE_SPACE_FOR_KEY_VALUE_PAIRS: usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;
const LEAF_NODE_MAX_KEY_VALUE_PAIRS: usize =
    LEAF_NODE_SPACE_FOR_KEY_VALUE_PAIRS / LEAF_NODE_KEY_VALUE_PAIR_SIZE;

#[derive(PartialEq)]
pub enum NodeType {
    Internal = 1,
    Leaf = 2,
    Unknown,
}

impl From<u8> for NodeType {
    fn from(orig: u8) -> Self {
        match orig {
            0x1 => return NodeType::Internal,
            0x2 => return NodeType::Leaf,
            _ => return NodeType::Unknown,
        };
    }
}

/// Node represents a node in the BTree occupied by a single page in memory.
pub struct Node {
    pub node_type: NodeType,
    pub offset: usize,
    pub parent_pointer_offset: usize,
    pub is_root: bool,
}

impl Node {
    pub fn new(
        node_type: NodeType,
        offset: usize,
        parent_pointer_offset: usize,
        is_root: bool,
    ) -> Node {
        Node {
            node_type,
            offset,
            parent_pointer_offset,
            is_root,
        }
    }
    pub fn page_to_node(offset: usize, page: &[u8]) -> Result<Node, Error> {
        let is_root = Node::is_root(page[IS_ROOT_OFFSET]);
        let node_type = NodeType::from(page[NODE_TYPE_OFFSET + NODE_TYPE_SIZE]);
        if node_type == NodeType::Unknown {
            return Err(Error::UnexpectedError);
        }
        let parent_pointer_offset = offset + PARENT_POINTER_OFFSET;

        return Ok(Node::new(node_type, offset, parent_pointer_offset, is_root));
    }

    fn is_root(b: u8) -> bool {
        match b {
            0x1 => true,
            _ => false,
        }
    }
}
