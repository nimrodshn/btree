use std::convert::From;

// NodeType Represents different node types in the BTree.
#[derive(PartialEq)]
pub enum NodeType {
    Internal = 1,
    Leaf = 2,
    Unknown,
}

// Converts a byte to a NodeType.
impl From<u8> for NodeType {
    fn from(orig: u8) -> Self {
        match orig {
            0x01 => return NodeType::Internal,
            0x02 => return NodeType::Leaf,
            _ => return NodeType::Unknown,
        };
    }
}

// Converts a NodeType to a byte.
impl From<NodeType> for u8 {
    fn from(orig: NodeType) -> Self {
        match orig {
            NodeType::Internal => 0x01,
            NodeType::Leaf => 0x02,
            NodeType::Unknown => 0x03,
        }
    }
}
