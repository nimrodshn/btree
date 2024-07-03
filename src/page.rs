use crate::error::Error;
use crate::node::Node;
use crate::node_type::{Key, NodeType, Offset};
use crate::page_layout::{
    ToByte, INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_OFFSET,
    INTERNAL_NODE_NUM_CHILDREN_SIZE, IS_ROOT_OFFSET, KEY_SIZE, LEAF_NODE_HEADER_SIZE,
    LEAF_NODE_NUM_PAIRS_OFFSET, LEAF_NODE_NUM_PAIRS_SIZE, NODE_TYPE_OFFSET, PAGE_SIZE,
    PARENT_POINTER_OFFSET, PARENT_POINTER_SIZE, PTR_SIZE, VALUE_SIZE,
};
use std::convert::TryFrom;

/// Value is a wrapper for a value in the page.
pub struct Value(pub usize);

/// Page is a wrapper for a single page of memory
/// providing some helpful helpers for quick access.
pub struct Page {
    data: Box<[u8; PAGE_SIZE]>,
}

impl Page {
    pub fn new(data: [u8; PAGE_SIZE]) -> Page {
        Page {
            data: Box::new(data),
        }
    }

    /// write_value_at_offset writes a given value (as BigEndian) at a certain offset
    /// overriding values at that offset.
    pub fn write_value_at_offset(&mut self, offset: usize, value: usize) -> Result<(), Error> {
        if offset > PAGE_SIZE - PTR_SIZE {
            return Err(Error::UnexpectedError);
        }
        let bytes = value.to_be_bytes();
        self.data[offset..offset + PTR_SIZE].clone_from_slice(&bytes);
        Ok(())
    }

    /// get_value_from_offset Fetches a value calculated as BigEndian, sized to usize.
    /// This function may error as the value might not fit into a usize.
    pub fn get_value_from_offset(&self, offset: usize) -> Result<usize, Error> {
        let bytes = &self.data[offset..offset + PTR_SIZE];
        let Value(res) = Value::try_from(bytes)?;
        Ok(res)
    }

    /// insert_bytes_at_offset pushes #size bytes from offset to end_offset
    /// inserts #size bytes from given slice.
    pub fn insert_bytes_at_offset(
        &mut self,
        bytes: &[u8],
        offset: usize,
        end_offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        // This Should not occur - better verify.
        if end_offset + size > self.data.len() {
            return Err(Error::UnexpectedError);
        }
        for idx in (offset..=end_offset).rev() {
            self.data[idx + size] = self.data[idx]
        }
        self.data[offset..offset + size].clone_from_slice(bytes);
        Ok(())
    }

    /// write_bytes_at_offset write bytes at a certain offset overriding previous values.
    pub fn write_bytes_at_offset(
        &mut self,
        bytes: &[u8],
        offset: usize,
        size: usize,
    ) -> Result<(), Error> {
        self.data[offset..offset + size].clone_from_slice(bytes);
        Ok(())
    }

    /// get_ptr_from_offset Fetches a slice of bytes from certain offset and of certain size.
    pub fn get_ptr_from_offset(&self, offset: usize, size: usize) -> &[u8] {
        &self.data[offset..offset + size]
    }

    /// get_data returns the underlying array.
    pub fn get_data(&self) -> [u8; PAGE_SIZE] {
        *self.data
    }
}

/// Implement TryFrom<Box<Node>> for Page allowing for easier
/// serialization of data from a Node to an on-disk formatted page.
impl TryFrom<&Node> for Page {
    type Error = Error;
    fn try_from(node: &Node) -> Result<Page, Error> {
        let mut data: [u8; PAGE_SIZE] = [0x00; PAGE_SIZE];
        // is_root byte
        data[IS_ROOT_OFFSET] = node.is_root.to_byte();

        // node_type byte
        data[NODE_TYPE_OFFSET] = u8::from(&node.node_type);

        // parent offest
        if !node.is_root {
            match node.parent_offset {
                Some(Offset(parent_offset)) => data
                    [PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
                    .clone_from_slice(&parent_offset.to_be_bytes()),
                // Expected an offset of an inner / leaf node.
                None => return Err(Error::UnexpectedError),
            };
        }

        match &node.node_type {
            NodeType::Internal(child_offsets, keys) => {
                data[INTERNAL_NODE_NUM_CHILDREN_OFFSET
                    ..INTERNAL_NODE_NUM_CHILDREN_OFFSET + INTERNAL_NODE_NUM_CHILDREN_SIZE]
                    .clone_from_slice(&child_offsets.len().to_be_bytes());

                let mut page_offset = INTERNAL_NODE_HEADER_SIZE;
                for Offset(child_offset) in child_offsets {
                    data[page_offset..page_offset + PTR_SIZE]
                        .clone_from_slice(&child_offset.to_be_bytes());
                    page_offset += PTR_SIZE;
                }

                for Key(key) in keys {
                    let key_bytes = key.as_bytes();
                    let mut raw_key: [u8; KEY_SIZE] = [0x00; KEY_SIZE];
                    if key_bytes.len() > KEY_SIZE {
                        return Err(Error::KeyOverflowError);
                    } else {
                        for (i, byte) in key_bytes.iter().enumerate() {
                            raw_key[i] = *byte;
                        }
                    }
                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(&raw_key);
                    page_offset += KEY_SIZE
                }
            }
            NodeType::Leaf(kv_pairs) => {
                // num of pairs
                data[LEAF_NODE_NUM_PAIRS_OFFSET
                    ..LEAF_NODE_NUM_PAIRS_OFFSET + LEAF_NODE_NUM_PAIRS_SIZE]
                    .clone_from_slice(&kv_pairs.len().to_be_bytes());

                let mut page_offset = LEAF_NODE_HEADER_SIZE;
                for pair in kv_pairs {
                    let key_bytes = pair.key.as_bytes();
                    let mut raw_key: [u8; KEY_SIZE] = [0x00; KEY_SIZE];
                    if key_bytes.len() > KEY_SIZE {
                        return Err(Error::KeyOverflowError);
                    } else {
                        for (i, byte) in key_bytes.iter().enumerate() {
                            raw_key[i] = *byte;
                        }
                    }
                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(&raw_key);
                    page_offset += KEY_SIZE;

                    let value_bytes = pair.value.as_bytes();
                    let mut raw_value: [u8; VALUE_SIZE] = [0x00; VALUE_SIZE];
                    if value_bytes.len() > VALUE_SIZE {
                        return Err(Error::ValueOverflowError);
                    } else {
                        for (i, byte) in value_bytes.iter().enumerate() {
                            raw_value[i] = *byte;
                        }
                    }
                    data[page_offset..page_offset + VALUE_SIZE].clone_from_slice(&raw_value);
                    page_offset += VALUE_SIZE;
                }
            }
            NodeType::Unexpected => return Err(Error::UnexpectedError),
        }

        Ok(Page::new(data))
    }
}

/// Attempts to convert a slice to an array of a fixed size (PTR_SIZE),
/// and then return the BigEndian value of the byte array.
impl TryFrom<&[u8]> for Value {
    type Error = Error;

    fn try_from(arr: &[u8]) -> Result<Self, Self::Error> {
        if arr.len() > PTR_SIZE {
            return Err(Error::TryFromSliceError("Unexpected Error: Array recieved is larger than the maximum allowed size of: 4096B."));
        }

        let mut truncated_arr = [0u8; PTR_SIZE];
        for (i, item) in arr.iter().enumerate() {
            truncated_arr[i] = *item;
        }

        Ok(Value(usize::from_be_bytes(truncated_arr)))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[test]
    fn node_to_page_works_for_leaf_node() -> Result<(), Error> {
        use crate::node::Node;
        use crate::node_type::{KeyValuePair, NodeType};
        use crate::page::Page;
        use std::convert::TryFrom;

        let some_leaf = Node::new(
            NodeType::Leaf(vec![
                KeyValuePair::new("foo".to_string(), "bar".to_string()),
                KeyValuePair::new("lebron".to_string(), "james".to_string()),
                KeyValuePair::new("ariana".to_string(), "grande".to_string()),
            ]),
            true,
            None,
        );

        // Serialize data.
        let page = Page::try_from(&some_leaf)?;
        // Deserialize back the page.
        let res = Node::try_from(page)?;

        assert_eq!(res.is_root, some_leaf.is_root);
        assert_eq!(res.node_type, some_leaf.node_type);
        assert_eq!(res.parent_offset, some_leaf.parent_offset);
        Ok(())
    }

    #[test]
    fn node_to_page_works_for_internal_node() -> Result<(), Error> {
        use crate::node::Node;
        use crate::node_type::{Key, NodeType, Offset};
        use crate::page::Page;
        use crate::page_layout::PAGE_SIZE;
        use std::convert::TryFrom;

        let internal_node = Node::new(
            NodeType::Internal(
                vec![
                    Offset(PAGE_SIZE),
                    Offset(PAGE_SIZE * 2),
                    Offset(PAGE_SIZE * 3),
                    Offset(PAGE_SIZE * 4),
                ],
                vec![
                    Key("foo bar".to_string()),
                    Key("lebron".to_string()),
                    Key("ariana".to_string()),
                ],
            ),
            true,
            None,
        );

        // Serialize data.
        let page = Page::try_from(&internal_node)?;
        // Deserialize back the page.
        let res = Node::try_from(page)?;

        assert_eq!(res.is_root, internal_node.is_root);
        assert_eq!(res.node_type, internal_node.node_type);
        assert_eq!(res.parent_offset, internal_node.parent_offset);
        Ok(())
    }
}
