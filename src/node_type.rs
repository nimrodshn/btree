use crate::key_value_pair::KeyValuePair;
use std::cmp::Eq;
use std::convert::From;

#[derive(Clone, Eq, PartialEq)]
pub struct Offset(pub usize);

#[derive(Clone, Eq, PartialEq)]
pub struct Key(pub String);

// NodeType Represents different node types in the BTree.
#[derive(PartialEq, Eq, Clone)]
pub enum NodeType {
    /// Internal nodes contain a vector of pointers to their children and a vector of keys.
    Internal(Vec<Offset>, Vec<Key>),

    /// Leaf nodes contain a vector of Keys and values.
    Leaf(Vec<KeyValuePair>),

    Unexpected,
}

// Converts a byte to a NodeType.
impl From<u8> for NodeType {
    fn from(orig: u8) -> NodeType {
        match orig {
            0x01 => NodeType::Internal(Vec::<Offset>::new(), Vec::<Key>::new()),
            0x02 => NodeType::Leaf(Vec::<KeyValuePair>::new()),
            _ => NodeType::Unexpected,
        }
    }
}

// Converts a NodeType to a byte.
impl From<&NodeType> for u8 {
    fn from(orig: &NodeType) -> u8 {
        match orig {
            NodeType::Internal(_, _) => 0x01,
            NodeType::Leaf(_) => 0x02,
            NodeType::Unexpected => 0x03,
        }
    }
}
