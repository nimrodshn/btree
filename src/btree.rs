use crate::error::Error;
use crate::node::Node;
use crate::node_type::{Key, KeyValuePair, NodeType, Offset};
use crate::page::Page;
use crate::pager::Pager;
use std::convert::TryFrom;
use std::path::Path;

/// B+Tree properties.
pub const MAX_BRANCHING_FACTOR: usize = 200;
pub const MIN_BRANCHING_FACTOR: usize = 100;
pub const NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

/// BTree struct represents an on-disk B+tree.
/// Each node is persisted in the table file, the leaf nodes contain the values.
pub struct BTree {
    pager: Pager,
    b: usize,
    root_offset: Offset,
}

impl BTree {
    fn new(path: &Path, b: usize) -> Result<BTree, Error> {
        let mut pager = Pager::new(&path)?;
        let root = Node::new(NodeType::Leaf(Vec::<KeyValuePair>::new()), true, None);
        let root_offset = pager.write_page(Page::try_from(&root)?)?;
        Ok(BTree {
            pager,
            b,
            root_offset,
        })
    }

    fn is_node_full(&self, node: &Node) -> Result<bool, Error> {
        match &node.node_type {
            NodeType::Leaf(pairs) => Ok(pairs.len() == (2 * self.b - 1)),
            NodeType::Internal(_, keys) => Ok(keys.len() == (2 * self.b - 1)),
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// insert a key value pair possibly splitting nodes along the way.
    pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        let root_page = self.pager.get_page(&self.root_offset)?;
        let mut root = Node::try_from(root_page)?;
        if self.is_node_full(&root)? {
            let mut old_root = &mut root;
            let old_root_offset = self.root_offset.clone();
            let mut new_root = Node::new(
                NodeType::Internal(vec![], vec![]),
                true,
                None,
            );
            // write the new root to disk.
            let new_root_offset = self.pager.write_page(Page::try_from(&new_root)?)?;
            // Set the current roots parent to the new root.
            old_root.parent_offset = Some(new_root_offset.clone());
            // update the root offset.
            self.root_offset = new_root_offset;
            // split the old root.
            let (median, sibling) = old_root.split(self.b)?;
            // Write the old root with its new data to disk.
            self.pager
                .write_page_at_offset(Page::try_from(&*old_root)?, &old_root_offset)?;
            // Write the newly created sibling to disk.
            let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
            // Update the new root with its children and key.
            new_root.node_type =
                NodeType::Internal(vec![sibling_offset, old_root_offset], vec![median]);
            // Write the new_root to disk.
            self.pager
                .write_page_at_offset(Page::try_from(&new_root)?, &self.root_offset)?;
        }
        self.insert_non_full(&mut root, self.root_offset.clone(), kv)
    }

    /// insert_non_full (recoursively) finds a node rooted at a given non-full node.
    /// to insert a given kv pair.
    fn insert_non_full(
        &mut self,
        node: &mut Node,
        node_offset: Offset,
        kv: KeyValuePair,
    ) -> Result<(), Error> {
        match &mut node.node_type {
            NodeType::Leaf(ref mut pairs) => {
                let idx = pairs.binary_search(&kv).unwrap_or_else(|x| x);
                pairs.insert(idx, kv);
                self.pager
                    .write_page_at_offset(Page::try_from(&*node)?, &node_offset)
            }
            NodeType::Internal(ref mut children, ref mut keys) => {
                let idx = keys
                    .binary_search(&Key(kv.key.clone()))
                    .unwrap_or_else(|x| x);
                let child_offset = children.get(idx).ok_or(Error::UnexpectedError)?.clone();
                let child_page = self.pager.get_page(&child_offset)?;
                let mut child = Node::try_from(child_page)?;
                if self.is_node_full(&child)? {
                    let (median, sibling) = child.split(self.b)?;
                    self.pager
                        .write_page_at_offset(Page::try_from(&child)?, &child_offset)?;
                    // Write the newly created sibling to disk.
                    let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
                    children.insert(idx, sibling_offset);
                    keys.insert(idx, median);

                    // Write the parent page to disk.
                    self.pager
                        .write_page_at_offset(Page::try_from(&*node)?, &node_offset)?;
                }
                self.insert_non_full(&mut child, child_offset, kv)
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// search searches for a specific key in the BTree.
    pub fn search(&mut self, key: String) -> Result<KeyValuePair, Error> {
        let root_page = self.pager.get_page(&self.root_offset)?;
        let root = Node::try_from(root_page)?;
        self.search_node(root, &key)
    }

    /// search_node recursively searches a sub tree rooted at node for a key
    /// using a Pager to request pages as it traverses the subtree.
    /// if we have traveresed all the way to the leaves and the key was not found the method
    /// returns the leaf node and None indicating the key was not found,
    /// otherwise, continues recursively or return the appropriate error.
    fn search_node(&mut self, node: Node, search: &str) -> Result<KeyValuePair, Error> {
        match node.node_type {
            NodeType::Internal(children, keys) => {
                let idx = keys
                    .binary_search(&Key(search.to_string()))
                    .unwrap_or_else(|x| x);
                // Retrieve child page from disk and deserialize.
                let child_offset = children.get(idx).ok_or(Error::UnexpectedError)?;
                let page = self.pager.get_page(child_offset)?;
                let child_node = Node::try_from(page)?;
                self.search_node(child_node, search)
            }
            NodeType::Leaf(pairs) => {
                if let Ok(idx) =
                    pairs.binary_search_by_key(&search.to_string(), |pair| pair.key.clone())
                {
                    return Ok(pairs[idx].clone());
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
        use crate::btree::BTree;
        use crate::node_type::KeyValuePair;
        use std::path::Path;

        let mut btree = BTree::new(Path::new("/tmp/db"), 2)?;
        btree.insert(KeyValuePair::new("a".to_string(), "shalom".to_string()))?;
        btree.insert(KeyValuePair::new("b".to_string(), "hello".to_string()))?;
        btree.insert(KeyValuePair::new("c".to_string(), "marhaba".to_string()))?;

        let mut kv = btree.search("b".to_string())?;
        assert_eq!(kv.key, "b");
        assert_eq!(kv.value, "hello");

        kv = btree.search("c".to_string())?;
        assert_eq!(kv.key, "c");
        assert_eq!(kv.value, "marhaba");
        Ok(())
    }

    #[test]
    fn insert_works() -> Result<(), Error> {
        use crate::btree::BTree;
        use crate::node_type::KeyValuePair;
        use std::path::Path;

        let mut btree = BTree::new(Path::new("/tmp/db"), 2)?;
        btree.insert(KeyValuePair::new("a".to_string(), "shalom".to_string()))?;
        btree.insert(KeyValuePair::new("b".to_string(), "hello".to_string()))?;
        btree.insert(KeyValuePair::new("c".to_string(), "marhaba".to_string()))?;
        btree.insert(KeyValuePair::new("d".to_string(), "olah".to_string()))?;

        let mut kv = btree.search("b".to_string())?;
        assert_eq!(kv.key, "b");
        assert_eq!(kv.value, "hello");

        kv = btree.search("d".to_string())?;
        assert_eq!(kv.key, "d");
        assert_eq!(kv.value, "olah");
        Ok(())
    }
}
