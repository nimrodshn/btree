use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::node::{Node,NodeType};
use crate::pager::Pager;

/// B+Tree properties.
pub const MAX_BRANCHING_FACTOR: usize = 200;
pub const MIN_BRANCHING_FACTOR: usize = 100;
pub const INTERNAL_NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

/// struct represents an on-disk (persisted) Btree implementation
/// Each node is persisted in itself where the leaf nodes contain the values.
/// This implementation works best when node size are large, say 200+ Kb.
pub struct BTree {
    root: Node,
    pager: Pager,
}

impl BTree {
    fn new(pager: Pager, root: Node) -> BTree {
        BTree { pager, root }
    }

    /// search searches for a specific key in the BTree.
    fn search(&mut self, key: String) -> Result<KeyValuePair, Error> {
        search_node(&mut self.pager, &self.root, key)
    }
}

/// search_node recursively searches a sub tree rooted at node for a key
/// using a Pager to request pages as it traverses the subtree.
fn search_node(pager: &mut Pager, node: &Node, search_key: String) -> Result<KeyValuePair, Error> {
    let keys = node.get_keys()?;
    for (i, key) in keys.iter().enumerate() {
        // If this is the case were at a leaf node.
        if *key == search_key {
            let kv_pairs = node.get_key_value_pairs()?;
            match kv_pairs.get(i) {
                None => return Err(Error::UnexpectedError),
                Some(kv) => return Ok(kv.clone()),
            };
        }
        if *key > search_key {
            let children_ptrs = node.get_children()?;
            let child_offset = match children_ptrs.get(i) {
                None => return Err(Error::UnexpectedError),
                Some(child_offset) => child_offset,
            };
            let child_node = Node::page_to_node(*child_offset, pager.get_page(*child_offset)?)?;
            return search_node(pager, &child_node, search_key);
        }
    }
    // reaching here means we have searched through all the keys.
    // if we have traveresed all the way to the leaves than return `KeyNotFound`,
    // otherwise, continue recursively or return the appropriate error.
    match node.node_type {
        NodeType::Leaf => return Err(Error::KeyNotFound),
        NodeType::Internal => {
            let children_ptrs = node.get_children()?;
            let child_offset = match children_ptrs.last() {
                None => return Err(Error::UnexpectedError),
                Some(child_offset) => child_offset,
            };
            let child_node = Node::page_to_node(*child_offset, pager.get_page(*child_offset)?)?;
            return search_node(pager, &child_node, search_key);
        }
        NodeType::Unknown => return Err(Error::UnexpectedError),
    };    
}
