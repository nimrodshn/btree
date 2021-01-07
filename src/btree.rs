use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::node::{Node, PageAndOffset};
use crate::node_type::NodeType;
use crate::pager::Pager;
use std::convert::TryFrom;
use std::sync::{Arc, RwLock};

/// B+Tree properties.
pub const MAX_BRANCHING_FACTOR: usize = 200;
pub const MIN_BRANCHING_FACTOR: usize = 100;
pub const NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

/// BTree struct represents an on-disk B+tree.
/// Each node is persisted in the table file, the leaf nodes contain the values.
pub struct BTree {
    root: Arc<RwLock<Node>>,
    pager: Pager,
}

impl BTree {
    fn new(pager: Pager, root: Node) -> BTree {
        BTree {
            pager,
            root: Arc::new(RwLock::new(root)),
        }
    }

    /// search searches for a specific key in the BTree.
    pub fn search(&mut self, key: String) -> Result<KeyValuePair, Error> {
        let (_, kv) = self.search_node(Arc::clone(&self.root), &key)?;
        match kv {
            Some(kv) => return Ok(kv),
            None => return Err(Error::KeyNotFound),
        }
    }

    /// insert a key value pair possibly splitting nodes along the way.
    pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        let (node, kv_pair_exists) = self.search_node(Arc::clone(&self.root), &kv.key)?;
        match kv_pair_exists {
            // Key already exists in the tree.
            Some(_) => return Err(Error::KeyAlreadyExists),
            None => (),
        };
        // add key to node here possibly splitting nodes along the way.
        let mut guarded_node = match node.write() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        let keys_len = guarded_node.get_keys_len()?;
        if keys_len < NODE_KEYS_LIMIT {
            // Add the new key value pair to the in-memory struct.
            guarded_node.add_key_value_pair(kv)?;
            // Write the corresponding page to file.
            return self
                .pager
                .write_page(&guarded_node.page, &guarded_node.offset);
        }
        self.split_node(Arc::clone(&node))?;
        Ok(())
    }

    /// search_node recursively searches a sub tree rooted at node for a key
    /// using a Pager to request pages as it traverses the subtree.
    /// if we have traveresed all the way to the leaves and the key was not found the method
    /// returns the leaf node and None indicating the key was not found,
    /// otherwise, continues recursively or return the appropriate error.
    fn search_node(
        &mut self,
        node: Arc<RwLock<Node>>,
        search_key: &String,
    ) -> Result<(Arc<RwLock<Node>>, Option<KeyValuePair>), Error> {
        let guarded_node = match node.read() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        let keys = guarded_node.get_keys()?;
        for (i, key) in keys.iter().enumerate() {
            // If this is the case were at a leaf node.
            if *key == *search_key {
                let kv_pairs = guarded_node.get_key_value_pairs()?;
                match kv_pairs.get(i) {
                    None => return Err(Error::UnexpectedError),
                    Some(kv) => return Ok((Arc::clone(&node), Some(kv.clone()))),
                };
            }
            if *key > *search_key {
                return self.traverse_or_return(Arc::clone(&node), i, search_key);
            }
        }
        self.traverse_or_return(Arc::clone(&node), keys.len(), search_key)
    }

    fn traverse_or_return(
        &mut self,
        node: Arc<RwLock<Node>>,
        index: usize,
        search_key: &String,
    ) -> Result<(Arc<RwLock<Node>>, Option<KeyValuePair>), Error> {
        let guarded_node = match node.read() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        match guarded_node.node_type {
            NodeType::Leaf => return Ok((Arc::clone(&node), None)),
            NodeType::Internal => {
                let children_ptrs = guarded_node.get_children()?;
                let child_offset = match children_ptrs.get(index) {
                    None => return Err(Error::UnexpectedError),
                    Some(child_offset) => child_offset,
                };
                let child_node = Node::try_from(PageAndOffset {
                    offset: *child_offset,
                    page_data: self.pager.get_page(*child_offset)?,
                })?;
                return self.search_node(Arc::new(RwLock::new(child_node)), search_key);
            }
            NodeType::Unknown => return Err(Error::UnexpectedError),
        };
    }

    fn split_node(&mut self, node: Arc<RwLock<Node>>) -> Result<(), Error> {
        let guarded_node = match node.write() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        let keys = guarded_node.get_keys()?;
        let mut parent_node = Node::try_from(PageAndOffset {
            offset: guarded_node.parent_offset,
            page_data: self.pager.get_page(guarded_node.parent_offset)?,
        })?;
        let median_key = &keys[keys.len() / 2];
        parent_node.add_key(median_key.to_string())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[test]
    fn search_works() -> Result<(), Error> {
        // TOOD: write this.
        Ok(())
    }

    #[test]
    fn insert_works() -> Result<(), Error> {
        // TOOD: write this.
        Ok(())
    }
}
