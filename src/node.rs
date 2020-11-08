use crate::btree::MAX_BRANCHING_FACTOR;
use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::page::{Page, PAGE_SIZE, PTR_SIZE};
use std::convert::TryFrom;
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
    pub parent_offset: usize,
    pub is_root: bool,
    pub offset: usize,
    pub page: Page,
}

// Node represents a node in the B-Tree.
impl Node {
    pub fn new(
        node_type: NodeType,
        parent_offset: usize,
        offset: usize,
        is_root: bool,
        page: Page,
    ) -> Node {
        Node {
            node_type,
            parent_offset,
            offset,
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

    /// get_keys returns a A result which contains a vector with the keys contained in the node.
    pub fn get_keys(&self) -> Result<Vec<String>, Error> {
        match self.node_type {
            NodeType::Internal => {
                let num_children = self
                    .page
                    .get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut result = Vec::<String>::new();
                let mut offset = INTERNAL_NODE_HEADER_SIZE + num_children * PTR_SIZE;
                // Number of keys is always one less than the number of children.
                let num_keys = num_children - 1;
                for _i in 1..=num_keys {
                    let key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += KEY_SIZE;
                    // Trim leading or trailing zeros.
                    result.push(key.trim_matches(char::from(0)).to_string());
                }
                return Ok(result);
            }
            NodeType::Leaf => {
                let mut res = Vec::<String>::new();
                let mut offset = LEAF_NODE_NUM_PAIRS_OFFSET;
                let num_keys_val_pairs = self.page.get_value_from_offset(offset)?;
                offset = LEAF_NODE_HEADER_SIZE;
                for _i in 1..=num_keys_val_pairs {
                    let key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    // Skip the values; keys and values are bunched up together.
                    offset += KEY_SIZE + VALUE_SIZE;
                    res.push(key.trim_matches(char::from(0)).to_string());
                }
                return Ok(res);
            }
            NodeType::Unknown => return Err(Error::UnexpectedError),
        };
    }

    /// add_key_value_pair adds a key value pair to self,
    /// Intended for Leaf nodes only.
    pub fn add_key_value_pair(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        match self.node_type {
            NodeType::Leaf => {
                let num_keys_val_pairs = self
                    .page
                    .get_value_from_offset(LEAF_NODE_NUM_PAIRS_OFFSET)?;
                let offset = LEAF_NODE_HEADER_SIZE + (KEY_SIZE + VALUE_SIZE) * num_keys_val_pairs;
                // Update number of key value pairs.
                self.page
                    .write_value_at_offset(LEAF_NODE_NUM_PAIRS_OFFSET, num_keys_val_pairs + 1)?;
                // Write the key.
                let key_raw = kv.key.as_bytes();
                self.page.write_bytes_at_offset(key_raw, offset, KEY_SIZE)?;
                // Write the value.
                let value_raw = kv.value.as_bytes();
                self.page
                    .write_bytes_at_offset(value_raw, offset + KEY_SIZE, VALUE_SIZE)?;
                Ok(())
            }
            _ => return Err(Error::UnexpectedError),
        }
    }

    /// add key adds a key to self,
    /// Intended for Internal nodes only.
    pub fn add_key(&mut self, key: String) -> Result<(), Error> {
        match self.node_type {
            NodeType::Internal => {
                let num_children = self
                    .page
                    .get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut offset = INTERNAL_NODE_HEADER_SIZE + (PTR_SIZE) * num_children;
                // Update number of children. (eq number of keys + 1)
                self.page
                    .write_value_at_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET, num_children + 1)?;
                // Find placement for new key.
                let num_keys = num_children - 1;
                let end_key_data = num_keys * KEY_SIZE;
                for _ in 1..=num_keys {
                    let key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                    let iter_key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    if iter_key.to_owned() >= key {
                        // Found the index to insert keys.
                        self.page.insert_bytes_at_offset(
                            key.as_bytes(),
                            offset,
                            end_key_data,
                            KEY_SIZE,
                        )?;
                        break;
                    }
                    offset += KEY_SIZE;
                }
            }
            _ => return Err(Error::UnexpectedError),
        }
        Ok(())
    }

    /// get_keys_len retrieves the number of keys in the node.
    pub fn get_keys_len(&self) -> Result<usize, Error> {
        match self.node_type {
            NodeType::Internal => {
                let num_children = self
                    .page
                    .get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let num_keys = num_children - 1;
                Ok(num_keys)
            }
            NodeType::Leaf => self.page.get_value_from_offset(LEAF_NODE_NUM_PAIRS_OFFSET),
            NodeType::Unknown => Err(Error::UnexpectedError),
        }
    }

    /// get_keys returns a A result which contains a vector with the keys contained in the node.
    pub fn find_key_value_pair(&self, key: String) -> Result<KeyValuePair, Error> {
        match self.node_type {
            NodeType::Leaf => {
                let kv_pairs = self.get_key_value_pairs()?;
                for kv_pair in kv_pairs {
                    if kv_pair.key == key {
                        return Ok(kv_pair);
                    }
                }
                Err(Error::KeyNotFound)
            }
            _ => return Err(Error::KeyNotFound),
        }
    }

    /// splits the current node returning the median key and the two split nodes.
    pub fn split(&self) -> Result<(String, Node, Node), Error> {
        match self.node_type {
            NodeType::Internal => {
                let num_children = self
                    .page
                    .get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut offset = INTERNAL_NODE_HEADER_SIZE + (PTR_SIZE) * num_children;

                let split_node_num_key = (num_children - 1) / 2;
                let mut left_node_keys = Vec::<&[u8]>::new();
                let mut right_node_keys = Vec::<&[u8]>::new();

                for _ in 1..split_node_num_key {
                    let key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                    left_node_keys.push(key_raw);
                    offset += KEY_SIZE;
                }

                let median_key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                let median_key = match str::from_utf8(median_key_raw) {
                    Ok(key) => key,
                    Err(_) => return Err(Error::UTF8Error),
                };

                offset += KEY_SIZE;
                for _ in 1..split_node_num_key {
                    let key_raw = self.page.get_ptr_from_offset(offset, KEY_SIZE);
                    right_node_keys.push(key_raw);
                    offset += KEY_SIZE;
                }

                // TODO: create the left and right nodes here. append them to the index file.
                // let left_node = Node::new(node_type: NodeType::Internal,
                //     offset: usize,
                //     parent_offset: usize,
                //     is_root: bool,
                //     page: Page)
                //Ok((String::from(median_key), ,))

                Err(Error::UnexpectedError)
            }
            NodeType::Leaf => Err(Error::UnexpectedError),
            NodeType::Unknown => Err(Error::UnexpectedError),
        }
    }
}

// NodeSpec is used to generate a Node by implementing TryFrom for thise type.
// It contains the raw information used to populate the nodes fields.
pub struct NodeSpec {
    pub page_data: [u8; PAGE_SIZE],
    pub offset: usize,
}

impl TryFrom<NodeSpec> for Node {
    type Error = Error;
    fn try_from(spec: NodeSpec) -> Result<Self, Self::Error> {
        let page = Page::new(spec.page_data);
        let is_root = match spec.page_data[IS_ROOT_OFFSET] {
            0x01 => true,
            _ => false,
        };

        let node_type = NodeType::from(spec.page_data[NODE_TYPE_OFFSET]);
        if node_type == NodeType::Unknown {
            return Err(Error::UnexpectedError);
        }

        let parent_pointer_offset = page.get_value_from_offset(PARENT_POINTER_OFFSET)?;

        return Ok(Node::new(
            node_type,
            parent_pointer_offset,
            spec.offset,
            is_root,
            page,
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::node::{
        Node, NodeSpec, INTERNAL_NODE_HEADER_SIZE, KEY_SIZE, LEAF_NODE_HEADER_SIZE, PTR_SIZE,
        VALUE_SIZE,
    };
    use crate::page::PAGE_SIZE;
    use std::convert::TryFrom;

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
        let node = Node::try_from(NodeSpec {
            offset: offset,
            page_data: page,
        })?;

        assert_eq!(node.is_root, true);
        assert_eq!(node.parent_offset, 0);

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
        let node = Node::try_from(NodeSpec {
            offset: offset,
            page_data: page,
        })?;
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
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, // 4096  (2nd Page)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, // 8192  (3rd Page)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, // 12288 (4th Page)
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
        let node = Node::try_from(NodeSpec {
            offset: offset,
            page_data: page,
        })?;
        let children = node.get_children()?;

        assert_eq!(children.len(), 3);
        for (i, child) in children.iter().enumerate() {
            assert_eq!(*child, PAGE_SIZE * (i + 1));
        }

        Ok(())
    }

    #[test]
    fn get_keys_work_for_internal_node() -> Result<(), Error> {
        const DATA_LEN: usize = INTERNAL_NODE_HEADER_SIZE + 3 * PTR_SIZE + 2 * KEY_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x01, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, // Number of children.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, // 4096  (2nd Page)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, // 8192  (3rd Page)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, // 12288 (4th Page)
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
        let node = Node::try_from(NodeSpec {
            offset: offset,
            page_data: page,
        })?;
        let keys = node.get_keys()?;
        assert_eq!(keys.len(), 2);

        let first_key = match keys.get(0) {
            Some(key) => key,
            None => return Err(Error::UnexpectedError),
        };
        assert_eq!(first_key, "hello");

        let second_key = match keys.get(1) {
            Some(key) => key,
            None => return Err(Error::UnexpectedError),
        };
        assert_eq!(second_key, "world");

        Ok(())
    }

    #[test]
    fn get_keys_work_for_leaf_node() -> Result<(), Error> {
        const DATA_LEN: usize = INTERNAL_NODE_HEADER_SIZE + 2 * KEY_SIZE + 2 * VALUE_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x02, // Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, // Number of Key-Value pairs.
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
            0x66, 0x6f, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // "foo"
            0x62, 0x61, 0x72, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // "bar"
        ];

        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];

        // Concatenate the two arrays; page_data and junk.
        let mut page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) {
            *to = *from
        }

        let offset = 0;
        let node = Node::try_from(NodeSpec {
            offset: offset,
            page_data: page,
        })?;

        let keys = node.get_keys()?;
        assert_eq!(keys.len(), 2);

        let first_key = match keys.get(0) {
            Some(key) => key,
            None => return Err(Error::UnexpectedError),
        };
        assert_eq!(first_key, "hello");

        let second_key = match keys.get(1) {
            Some(key) => key,
            None => return Err(Error::UnexpectedError),
        };
        assert_eq!(second_key, "foo");

        Ok(())
    }
}
