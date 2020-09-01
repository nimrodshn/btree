use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::page::{Page, PTR_SIZE, PAGE_SIZE};
use std::str;

/// Common Node header layout.
const IS_ROOT_SIZE: usize = 1;
const IS_ROOT_OFFSET: usize = 0;
const NODE_TYPE_SIZE: usize = 1;
const NODE_TYPE_OFFSET: usize = 1;
const PARENT_POINTER_OFFSET: usize = 2;
const PARENT_POINTER_SIZE: usize = PTR_SIZE;
const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

/// Leaf node header layout
const LEAF_NODE_NUM_PAIRS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const LEAF_NODE_NUM_PAIRS_SIZE: usize = PTR_SIZE;
const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_PAIRS_SIZE;

/// Leaf node body layout.
const KEY_SIZE_FIELD: usize = PTR_SIZE;
const VALUE_SIZE_FIELD: usize = PTR_SIZE;

/// Internal header layout
const INTERNAL_NODE_NUM_CHILDREN_SIZE: usize = PTR_SIZE;
const INTERNAL_NODE_NUM_CHILDREN_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
const INTERNAL_NODE_NUM_KEYS_SIZE: usize = PTR_SIZE;
const INTERNAL_NODE_NUM_KEYS_OFFSET: usize =
    INTERNAL_NODE_NUM_CHILDREN_OFFSET + INTERNAL_NODE_NUM_CHILDREN_SIZE;
const INTERNAL_NODE_HEADER_SIZE: usize =
    COMMON_NODE_HEADER_SIZE + INTERNAL_NODE_NUM_CHILDREN_SIZE + INTERNAL_NODE_NUM_KEYS_SIZE;

/// Internal node body layout
const CHILD_PTR_SIZE: usize = PTR_SIZE;

#[derive(PartialEq, Copy)]
pub enum NodeType {
    Internal = 1,
    Leaf = 2,
    Unknown,
}

impl Clone for NodeType {
    fn clone(&self) -> NodeType {
        *self
    }
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
            page,
        }
    }

    /// get_key_value_pairs returns a list of key value pairs in case of a leaf node,
    /// otherwise, returns an error.
    pub fn get_key_value_pairs(&self) -> Result<Vec<KeyValuePair>, Error> {
        match self.node_type {
            NodeType::Leaf => {
                let mut res = Vec::<KeyValuePair>::new();
                let mut offset = COMMON_NODE_HEADER_SIZE;
                let num_keys_val_pairs = self.page.get_value_from_offset(offset)?;

                offset = LEAF_NODE_HEADER_SIZE;

                for _i in 0..num_keys_val_pairs {
                    let key_raw = self.page.get_ptr_from_offset(offset);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    // Increment offset after getting the key.
                    offset = offset + PTR_SIZE + KEY_SIZE_FIELD;
                    let value_raw = self.page.get_ptr_from_offset(offset);
                    let value = match str::from_utf8(value_raw) {
                        Ok(val) => val,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    // Increment the offset after getting the value.
                    offset = offset + PTR_SIZE + VALUE_SIZE_FIELD;
                    res.push(KeyValuePair::new(key.to_string(), value.to_string()))
                }
                return Ok(res);
            }
            _ => return Err(Error::UnexpectedError),
        };
    }

    /// get_children returns the children of a certain node in case of an internal node,
    /// otherwise, returns an error.
    pub fn get_children(&self) -> Result<Vec<&[u8]>, Error> {
        match self.node_type {
            NodeType::Internal => {
                let num_children = self.page.get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut result = Vec::<&[u8]>::new();
                let offset = INTERNAL_NODE_HEADER_SIZE;
                for _i in 1..num_children {
                    let child_raw = self.page.get_ptr_from_offset(offset);
                    result.push(child_raw);
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

        return Ok(Node::new(node_type, offset, parent_pointer_offset, is_root, page));
    }
}

impl Clone for Node {
    fn clone(&self) -> Node {
        Node {
            is_root: self.is_root,
            node_type: self.node_type,
            offset: self.offset,
            parent_pointer_offset: self.parent_pointer_offset,
            page: self.page,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::node::{
        Node, INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_SIZE,
        INTERNAL_NODE_NUM_KEYS_SIZE, KEY_SIZE_FIELD, LEAF_NODE_HEADER_SIZE, PARENT_POINTER_OFFSET,
        VALUE_SIZE_FIELD,
    };
    use crate::page::PAGE_SIZE;
    use std::convert::TryInto;

    #[test]
    fn page_to_node_works() -> Result<(), Error> {
        const DATA_LEN: usize = LEAF_NODE_HEADER_SIZE + KEY_SIZE_FIELD + VALUE_SIZE_FIELD + 10;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x02, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // Key size.
            0x68, 0x65, 0x6c, 0x6c, 0x6f, // "hello"
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // Value size.
            0x77, 0x6f, 0x72, 0x6c, 0x64, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];
        let page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) { *to = *from };

        let offset = PAGE_SIZE * 2;
        let node = Node::page_to_node(offset, page)?;

        assert_eq!(node.is_root, true);
        assert_eq!(node.offset, offset);
        assert_eq!(node.parent_pointer_offset, offset + PARENT_POINTER_OFFSET);

        Ok(())
    }

    #[test]
    fn get_key_value_pairs_works() -> Result<(), Error> {
        const DATA_LEN: usize = LEAF_NODE_HEADER_SIZE + KEY_SIZE_FIELD + VALUE_SIZE_FIELD + 10;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x02, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // Key size.
            0x68, 0x65, 0x6c, 0x6c, 0x6f, // "hello"
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // Value size.
            0x77, 0x6f, 0x72, 0x6c, 0x64, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];
        let page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) { *to = *from };

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
}
