use crate::key_value_pair::KeyValuePair;
use crate::node_type::NodeType;
use crate::page::Page;
use crate::page_layout::{
    INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_OFFSET, INTERNAL_NODE_NUM_CHILDREN_SIZE,
    IS_ROOT_OFFSET, KEY_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_NUM_PAIRS_OFFSET,
    LEAF_NODE_NUM_PAIRS_SIZE, NODE_TYPE_OFFSET, PAGE_SIZE, PARENT_POINTER_OFFSET,
    PARENT_POINTER_SIZE, PTR_SIZE, VALUE_SIZE, ToByte,
};

pub trait PageBuilder {
    fn build(self) -> Page;
}

pub struct LeafNodePageBuilder {
    is_root: bool,
    node_type: NodeType,
    parent_offset: usize,
    key_value_pairs: Vec<KeyValuePair>,
}

impl LeafNodePageBuilder {
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

    pub fn key_value_pairs(&mut self, key_value_pairs: Vec<KeyValuePair>) -> &mut Self {
        self.key_value_pairs = key_value_pairs;
        self
    }
}

impl PageBuilder for LeafNodePageBuilder {
    fn build(self) -> Page {
        let mut data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        // is_root byte
        data[IS_ROOT_OFFSET] = self.is_root.to_byte();

        // node_type byte
        data[NODE_TYPE_OFFSET] = self.node_type.into();

        // parent pointer offest
        data[PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
            .clone_from_slice(&self.parent_offset.to_be_bytes());

        // num of pairs
        data[LEAF_NODE_NUM_PAIRS_OFFSET..LEAF_NODE_NUM_PAIRS_OFFSET + LEAF_NODE_NUM_PAIRS_SIZE]
            .clone_from_slice(&self.key_value_pairs.len().to_be_bytes());

        // key value pairs
        let mut offset = LEAF_NODE_HEADER_SIZE;
        for pair in self.key_value_pairs {
            data[offset..offset + KEY_SIZE].clone_from_slice(pair.key.as_bytes());
            offset += KEY_SIZE;

            data[offset..offset + VALUE_SIZE].clone_from_slice(pair.value.as_bytes());
            offset += VALUE_SIZE;
        }

        Page::new(data)
    }
}

pub struct InternalNodePageBuilder {
    is_root: bool,
    node_type: NodeType,
    parent_offset: usize,
    keys: Vec<String>,
    child_pointers: Vec<[u8; PTR_SIZE]>,
}

impl Default for InternalNodePageBuilder {
    fn default () -> InternalNodePageBuilder {
        InternalNodePageBuilder{
            is_root: false,
            parent_offset: 0,
            node_type: NodeType::Unknown,
            child_pointers: Vec::new(),
            keys: Vec::new(),
        }
    }
}

impl InternalNodePageBuilder {
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

    pub fn child_pointers(&mut self, child_pointers: Vec<[u8; PTR_SIZE]>) -> &mut Self {
        self.child_pointers = child_pointers;
        self
    }

    pub fn keys(&mut self, keys: Vec<String>) -> &mut Self {
        self.keys = keys;
        self
    }
}

impl PageBuilder for InternalNodePageBuilder {
    fn build(self) -> Page {
        let mut data: [u8; PAGE_SIZE] = [0; PAGE_SIZE];
        // is_root byte
        data[IS_ROOT_OFFSET] = self.is_root.to_byte();

        // node_type byte
        data[NODE_TYPE_OFFSET] = self.node_type.into();

        // parent pointer offest
        data[PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
            .clone_from_slice(&self.parent_offset.to_be_bytes());

        // num of pairs
        data[INTERNAL_NODE_NUM_CHILDREN_OFFSET
            ..INTERNAL_NODE_NUM_CHILDREN_OFFSET + INTERNAL_NODE_NUM_CHILDREN_SIZE]
            .clone_from_slice(&self.child_pointers.len().to_be_bytes());

        // child pointers
        let mut offset = INTERNAL_NODE_HEADER_SIZE;
        for ptr in self.child_pointers {
            data[offset..offset + PTR_SIZE].clone_from_slice(&ptr);
            offset += PTR_SIZE;
        }

        // keys
        for key in self.keys {
            data[offset..offset + KEY_SIZE].clone_from_slice(key.as_bytes());
            offset += KEY_SIZE
        }
        Page::new(data)
    }
}
