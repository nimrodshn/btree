use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use std::convert::TryInto;
use std::mem::size_of;
use std::str;

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
const LEAF_NODE_NUM_PAIRS_SIZE: usize = size_of::<usize>();
const LEAF_NODE_NUM_PAIRS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_PAIRS_OFFSET;

/// Leaf body layout.
const LEAF_NODE_SPACE_FOR_KEY_VALUE_PAIRS: usize = PAGE_SIZE - LEAF_NODE_HEADER_SIZE;
const KEY_SIZE_FIELD: usize = 12;
const VALUE_SIZE_FIELD: usize = 12;

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

    pub fn get_key_value_pairs(&self, page: &[u8]) -> Option<Vec<KeyValuePair>> {
        match self.node_type {
            NodeType::Leaf => {
                let num_keys_val_pairs = usize::from_be_bytes(to_usize(
                    &page[LEAF_NODE_NUM_PAIRS_OFFSET
                        ..LEAF_NODE_NUM_PAIRS_OFFSET + LEAF_NODE_NUM_PAIRS_SIZE],
                ));
                let mut res = Vec::<KeyValuePair>::new();
                let mut offset = LEAF_NODE_SPACE_FOR_KEY_VALUE_PAIRS;

                for _i in 1..num_keys_val_pairs {
                    let (key_raw, size) =
                        get_field_from_offset_and_size(page, offset, KEY_SIZE_FIELD);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return None,
                    };
                    offset = offset + size;

                    let (value_raw, size) =
                        get_field_from_offset_and_size(page, offset, VALUE_SIZE_FIELD);
                    let value = match str::from_utf8(value_raw) {
                        Ok(val) => val,
                        Err(_) => return None,
                    };
                    offset = offset + size;
                    res.push(KeyValuePair::new(key.to_string(), value.to_string()))
                }
                return Some(res);
            }
            _ => return None,
        };
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

// to_usize attempts to convert a silce of bytes to an array of usize size.
pub fn to_usize(slice: &[u8]) -> [u8; size_of::<usize>()] {
    slice.try_into().expect("slice with incorrect length")
}

/// get_field_from_offset_and_size returns the size and field (in bytes) from a given offset and field size.
fn get_field_from_offset_and_size(
    page: &[u8],
    mut offset: usize,
    field_size: usize,
) -> (&[u8], usize) {
    let size = usize::from_be_bytes(to_usize(&page[offset..offset + field_size]));
    offset = offset + field_size;
    (&page[offset..offset + size], size)
}
