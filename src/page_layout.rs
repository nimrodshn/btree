use crate::btree::MAX_BRANCHING_FACTOR;
use std::mem::size_of;

/// A single page size.
/// Each page represents a node in the BTree.
pub const PAGE_SIZE: usize = 4096;

pub const PTR_SIZE: usize = size_of::<usize>();

/// Common Node header layout (Ten bytes in total)
pub const IS_ROOT_SIZE: usize = 1;
pub const IS_ROOT_OFFSET: usize = 0;
pub const NODE_TYPE_SIZE: usize = 1;
pub const NODE_TYPE_OFFSET: usize = 1;
pub const PARENT_POINTER_OFFSET: usize = 2;
pub const PARENT_POINTER_SIZE: usize = PTR_SIZE;
pub const COMMON_NODE_HEADER_SIZE: usize = NODE_TYPE_SIZE + IS_ROOT_SIZE + PARENT_POINTER_SIZE;

/// Leaf node header layout (Eighteen bytes in total)
///
/// Space for keys and values: PAGE_SIZE - LEAF_NODE_HEADER_SIZE = 4096 - 18 = 4078 bytes.
/// Which leaves 4076 / keys_limit = 20 (ten for key and 10 for value).
pub const LEAF_NODE_NUM_PAIRS_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
pub const LEAF_NODE_NUM_PAIRS_SIZE: usize = PTR_SIZE;
pub const LEAF_NODE_HEADER_SIZE: usize = COMMON_NODE_HEADER_SIZE + LEAF_NODE_NUM_PAIRS_SIZE;

/// Internal header layout (Eighteen bytes in total)
///
// Space for children and keys: PAGE_SIZE - INTERNAL_NODE_HEADER_SIZE = 4096 - 18 = 4078 bytes.
pub const INTERNAL_NODE_NUM_CHILDREN_OFFSET: usize = COMMON_NODE_HEADER_SIZE;
pub const INTERNAL_NODE_NUM_CHILDREN_SIZE: usize = PTR_SIZE;
pub const INTERNAL_NODE_HEADER_SIZE: usize =
    COMMON_NODE_HEADER_SIZE + INTERNAL_NODE_NUM_CHILDREN_SIZE;

/// On a 64 bit machine the maximum space to keep all of the pointer
/// is 200 * 8 = 1600 bytes.
pub const MAX_SPACE_FOR_CHILDREN: usize = MAX_BRANCHING_FACTOR * PTR_SIZE;

/// This leaves the keys of an internal node 2478 bytes:
/// We use 1990 bytes for keys which leaves 488 bytes as junk.
/// This means each key is limited to 12 bytes. (2476 / keys limit = ~12)
/// Rounded down to 10 to accomodate the leave node.
pub const MAX_SPACE_FOR_KEYS: usize =
    PAGE_SIZE - INTERNAL_NODE_HEADER_SIZE - MAX_SPACE_FOR_CHILDREN;

/// Key, Value sizes.
pub const KEY_SIZE: usize = 10;
pub const VALUE_SIZE: usize = 10;

/// Wrappers for converting byte to bool and back.
/// The convention used throughout the index file is: one is true; otherwise - false.
pub trait FromByte {
    fn from_byte(&self) -> bool;
}

pub trait ToByte {
    fn to_byte(&self) -> u8;
}

impl FromByte for u8 {
    fn from_byte(&self) -> bool {
        match self {
            0x01 => true,
            _ => false,
        }
    }
}

impl ToByte for bool {
    fn to_byte(&self) -> u8 {
        match self {
            true => 0x01,
            false => 0x00,
        }
    }
}
