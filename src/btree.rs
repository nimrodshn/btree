use crate::error::Error;
use crate::key_value_pair::KeyValuePair;
use crate::node::Node;
use crate::node_type::{Key, NodeType, Offset};
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

    /// insert a key value pair possibly splitting nodes along the way.
    pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        Ok(())
    }

    /// search searches for a specific key in the BTree.
    pub fn search(&mut self, key: String) -> Result<KeyValuePair, Error> {
        self.search_node(Arc::clone(&self.root), &key)
    }

    /// search_node recursively searches a sub tree rooted at node for a key
    /// using a Pager to request pages as it traverses the subtree.
    /// if we have traveresed all the way to the leaves and the key was not found the method
    /// returns the leaf node and None indicating the key was not found,
    /// otherwise, continues recursively or return the appropriate error.
    fn search_node(
        &mut self,
        node: Arc<RwLock<Node>>,
        search: &str,
    ) -> Result<KeyValuePair, Error> {
        let guarded_node = match node.read() {
            Err(_) => return Err(Error::UnexpectedError),
            Ok(node) => node,
        };
        match &guarded_node.node_type {
            NodeType::Internal(children, keys) => {
                for (i, Key(key)) in keys.iter().enumerate() {
                    if search < key {
                        let Offset(child_offset) = match children.get(i) {
                            Some(offset) => offset,
                            None => return Err(Error::UnexpectedError),
                        };

                        let page = self.pager.get_page(*child_offset)?;
                        let child_node = Node::try_from(page)?;
                        return self.search_node(Arc::new(RwLock::from(child_node)), search);
                    }
                }
                Err(Error::KeyNotFound)
            }
            NodeType::Leaf(pairs) => {
                for kv_pair in pairs.iter() {
                    if kv_pair.key.eq(search) {
                        Some(kv_pair.clone());
                    }
                }
                Err(Error::KeyNotFound)
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[test]
    fn search_works() -> Result<(), Error> {
        // use crate::btree::BTree;
        // let btree = BTree::new();
        Ok(())
    }

    #[test]
    fn insert_works() -> Result<(), Error> {
        // TOOD: write this.
        Ok(())
    }
}
