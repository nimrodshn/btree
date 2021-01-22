use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::node_type::{Key, NodeType, Offset};
use crate::page::Page;
use crate::page_layout::{
    ToByte, INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_OFFSET,
    INTERNAL_NODE_NUM_CHILDREN_SIZE, IS_ROOT_OFFSET, KEY_SIZE, LEAF_NODE_HEADER_SIZE,
    LEAF_NODE_NUM_PAIRS_OFFSET, LEAF_NODE_NUM_PAIRS_SIZE, NODE_TYPE_OFFSET, PAGE_SIZE,
    PARENT_POINTER_OFFSET, PARENT_POINTER_SIZE, PTR_SIZE, VALUE_SIZE,
};

/// A helper builder for serializing in-mem Node object to a
/// Page object.
pub struct PageBuilder {
    is_root: bool,
    node_type: NodeType,
    parent_offset: usize,
}

impl PageBuilder {
    pub fn is_root(&mut self, is_root: bool) -> &mut Self {
        self.is_root = is_root;
        self
    }

    pub fn node_type(&mut self, node_type: NodeType) -> &mut Self {
        self.node_type = node_type;
        self
    }

    pub fn parent_offset(&mut self, parent_offset: usize) -> &mut Self {
        self.parent_offset = parent_offset;
        self
    }

    pub fn build(self) -> Result<Page, Error> {
        let mut data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        // is_root byte
        data[IS_ROOT_OFFSET] = self.is_root.to_byte();

        // node_type byte
        data[NODE_TYPE_OFFSET] = u8::from(&self.node_type);

        // parent pointer offest
        data[PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
            .clone_from_slice(&self.parent_offset.to_be_bytes());

        match self.node_type {
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
                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(key.as_bytes());
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
                    data[page_offset..page_offset + KEY_SIZE].clone_from_slice(pair.key.as_bytes());
                    page_offset += KEY_SIZE;

                    data[page_offset..page_offset + VALUE_SIZE]
                        .clone_from_slice(pair.value.as_bytes());
                    page_offset += VALUE_SIZE;
                }
            }
            NodeType::Unexpected => return Err(Error::UnexpectedError),
        }

        Ok(Page::new(data))
    }
}
