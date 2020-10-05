use crate::btree::MAX_BRANCHING_FACTOR;
use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::page::{Page, PAGE_SIZE, PTR_SIZE};
use std::str;

/// Common Node header layout (Ten bytes in total)
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = 0;
const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 1;
const PARENT_POINTER_OFFSET: usize = 2;
const PARENT_POINTER_SIZE: usize = PTR_SIZE;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

/// Leaf node header layout (Eighteen bytes in total)
///
/// Space for keys and values: PAGE_SIZE - LEAF_NODE_HEADER_SIZE = 4096 - 18 = 4076 bytes.
/// Which leaves 4076 / keys_limit = 20 (ten for key and 10 for value).
const LEAF_NODE_NUM_PAIRS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_NUM_PAIRS_SIZE: usize = PTR_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_PAIRS_SIZE;

/// Internal header layout (Eighteen bytes in total)
///
// Space for children and keys: PAGE_SIZE - INTERNAL_NODE_HEADER_SIZE = 4096 - 18 = 4076 bytes.
const INTERNAL_NODE_NUM_CHILDREN_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const INTERNAL_NODE_NUM_CHILDREN_SIZE: usize = PTR_SIZE;
const INTERNAL_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + INTERNAL_NODE_NUM_CHILDREN_SIZE;

/// On a 64 bit machine the maximum space to keep all of the pointer
/// is 200 * 8 = 1600 bytes.
const MAX_SPACE_FOR_CHILDREN: usize = MAX_BRANCHING_FACTOR * PTR_SIZE;

/// This leaves the keys of an internal node 2476 bytes:
/// We use 2388 bytes for keys which leaves 88 bytes as junk.
/// This means each key is limited to 12 bytes. (2476 / keys limit = ~12)
/// Rounded down to 10 to accomodate the leave node.
const MAX_SPACE_FOR_KEYS: usize = PAGE_SIZE - INTERNAL_NODE_HEADER_SIZE - MAX_SPACE_FOR_CHILDREN;

/// Key, Value sizes.
const KEY_SIZE: usize = 10;
const VALUE_SIZE: usize = 10;

#[derive(PartialEq)]
pub enum NodeType {
    Internal = 1,
    Leaf = 2,
    Unknown,
}

// Casts a byte to a NodeType.
impl From<u8> for NodeType {
    fn from(orig: u8) -> Self {
        match orig {
            0x01 => return NodeType::Internal,
            0x02 => return NodeType::Leaf,
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
    pub page: Page,
}

impl Node {
    pub fn new(
        node_type: NodeType,
        offset: usize,
        parent_pointer_offset: usize,
        is_root: bool,
        page: Page,
    ) -> Node {
        Node {
            node_type,
            offset,
            parent_pointer_offset,
            is_root,
            page: page,
        }
    }

    /// get_key_value_pairs returns a list of key value pairs in case of a leaf node,
    /// otherwise, returns an error.
    pub fn get_key_value_pairs(&self) -> Result<Vec<KeyValuePair>, Error> {
        match self.node_type {
            NodeType::Leaf => {
                let mut res = Vec::<KeyValuePair>::new();
                let mut offset = LEAF_NODE_NUM_PAIRS_OFFSET;
                let num_keys_val_pairs = self.page.get_value_from_offset(offset)?;

                offset = LEAF_NODE_HEADER_SIZE;

                for _i in 0..num_keys_val_pairs {
                    let key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += KEY_SIZE;

                    let value_raw = self.page.get_ptr_from_offset(offset, VALUE_SIZE);
                    let value = match str::from_utf8(value_raw) {
                        Ok(val) => val,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += VALUE_SIZE;

                    // Trim leading or trailing zeros.
                    res.push(KeyValuePair::new(
                        key.trim_matches(char::from(0)).to_string(),
                        value.trim_matches(char::from(0)).to_string(),
                    ))
                }
                return Ok(res);
            }
            _ => return Err(Error::UnexpectedError),
        };
    }

    /// get_children returns a *vector of offsets in the index file* to children of a certain node in case of an internal node,
    /// otherwise, returns an error.
    pub fn get_children(&self) -> Result<Vec<usize>, Error> {
        match self.node_type {
            NodeType::Internal => {
                let num_children = self
                    .page
                    .get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut result = Vec::<usize>::new();
                let mut offset = INTERNAL_NODE_HEADER_SIZE;
                for _i in 1..=num_children {
                    let child_offset = self.page.get_value_from_offset(offset)?;
                    result.push(child_offset);
                    offset += PTR_SIZE;
                }
                return Ok(result);
            }
            _ => return Err(Error::UnexpectedError),
        };
    }

    fn is_root(b: u8) -> bool {
        match b {
            0x01 => true,
            _ => false,
        }
    }

    // page_to_node converts a raw page of memory to an in-memory node.
    pub fn page_to_node(offset: usize, page: [u8; PAGE_SIZE]) -> Result<Node, Error> {
        let is_root = Node::is_root(page[IS_ROOT_OFFSET]);
        let node_type = NodeType::from(page[NODE_TYPE_OFFSET]);
        if node_type == NodeType::Unknown {
            return Err(Error::UnexpectedError);
        }

        let page = Page::new(page);
        let parent_pointer_offset = offset + PARENT_POINTER_OFFSET;

        return Ok(Node::new(
            node_type,
            offset,
            parent_pointer_offset,
            is_root,
            page,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::node::{
        Node, INTERNAL_NODE_HEADER_SIZE,PTR_SIZE, KEY_SIZE, LEAF_NODE_HEADER_SIZE, PARENT_POINTER_OFFSET,
        VALUE_SIZE,
    };
    use crate::page::PAGE_SIZE;

    #[test]
    fn page_to_node_works() -> Result<(), Error> {
        const DATA_LEN: usize = LEAF_NODE_HEADER_SIZE + KEY_SIZE + VALUE_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x02, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];
        let mut page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) {
            *to = *from
        }

        let offset = PAGE_SIZE * 2;
        let node = Node::page_to_node(offset, page)?;

        assert_eq!(node.is_root, true);
        assert_eq!(node.offset, offset);
        assert_eq!(node.parent_pointer_offset, offset + PARENT_POINTER_OFFSET);

        Ok(())
    }

    #[test]
    fn get_key_value_pairs_works() -> Result<(), Error> {
        const DATA_LEN: usize = LEAF_NODE_HEADER_SIZE + KEY_SIZE + VALUE_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x02, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];
        let mut page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) {
            *to = *from
        }

        let offset = PAGE_SIZE * 2;
        let node = Node::page_to_node(offset, page)?;
        let kv = node.get_key_value_pairs()?;

        assert_eq!(kv.len(), 1);
        let first_kv = match kv.get(0) {
            Some(kv) => kv,
            None => return Err(Error::UnexpectedError),
        };

        assert_eq!(first_kv.key, "hello");
        assert_eq!(first_kv.value, "world");

        Ok(())
    }

    #[test]
    fn get_children_works() -> Result<(), Error> {
        const DATA_LEN: usize = INTERNAL_NODE_HEADER_SIZE + 3 * PTR_SIZE + 2 * KEY_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x01, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, // Number of children.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, // 4096
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, // 8192
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, // 12288
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];

        // Concatenate the two arrays; page_data and junk.
        let mut page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) {
            *to = *from
        }

        let offset = 0;
        let node = Node::page_to_node(offset, page)?;
        let children = node.get_children()?;

        assert_eq!(children.len(), 3);
        for (i, child) in children.iter().enumerate() {
            assert_eq!(*child, PAGE_SIZE * (i+1));
        }
        
        Ok(())
    }
}
