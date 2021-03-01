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
        let root = Box::new(Node::new(
            NodeType::Leaf(Vec::<KeyValuePair>::new()),
            true,
            None,
        ));
        let root_offset = pager.write_page(Page::try_from(root)?)?;
        Ok(BTree {
            pager,
            b,
            root_offset,
        })
    }

    fn is_node_full(&self, node: Box<Node>) -> Result<bool, Error> {
        match node.node_type {
            NodeType::Leaf(pairs) => Ok(pairs.len() == (2 * self.b - 1)),
            NodeType::Internal(_, keys) => Ok(keys.len() == (2 * self.b - 1)),
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// insert a key value pair possibly splitting nodes along the way.
    pub fn insert(&mut self, kv: KeyValuePair) -> Result<(), Error> {
        let root_page = self.pager.get_page(self.root_offset.clone())?;
        let root = Box::new(Node::try_from(root_page)?);
        let is_full = self.is_node_full(root.clone())?;
        if is_full {
            let (child, child_offset) = self.allocate_new_root(root.clone())?;
            self.split_child(root.clone(), self.root_offset.clone(), child, child_offset)?;
        }
        self.insert_non_full(root, self.root_offset.clone(), kv)
    }

    /// split_child splits a child in two, moving the median key to its parents
    /// as well as adding the newly created sibling as another child to the parent created.
    fn split_child(
        &mut self,
        mut parent: Box<Node>,
        parent_offset: Offset,
        child: Box<Node>,
        child_offset: Offset,
    ) -> Result<(), Error> {
        let (median, sibling) = self.create_sibling_from_node(child.clone())?;
        // Write the child with its new data to disk.
        self.pager
        .write_page_at_offset(Page::try_from(child)?, child_offset.clone())?;
        
        // Write the newly created sibling to disk.
        let sibling_offset = self.pager.write_page(Page::try_from(sibling)?)?;
        
        // Update the parent with the new key and child.
        match parent.node_type {
            NodeType::Internal(ref mut parent_children, ref mut parent_keys) => {
                parent_children.push(sibling_offset);
                parent_keys.push(median);
            },
            NodeType::Leaf(_) => {
                // This can only happen when the root is split.
                parent.node_type = NodeType::Internal(vec![sibling_offset, child_offset], vec![median]);  
            }
            _ => return Err(Error::UnexpectedError),
        }

        // Write the parent page to disk.
        self.pager
            .write_page_at_offset(Page::try_from(parent)?, parent_offset)?;

        Ok(())
    }

    /// create_sibling_from_node creates a sibling node from a given node
    /// by splitting the node in two. In addition it return the median key of the original node.
    fn create_sibling_from_node(&mut self, node: Box<Node>) -> Result<(Key, Box<Node>), Error> {
        match node.node_type {
            NodeType::Internal(mut children, mut keys) => {
                let mut sibling_keys = Vec::<Key>::new();
                let mut sibling_children = Vec::<Offset>::new();
                // Populate siblings keys.
                for _i in 1..(self.b - 1) {
                    let key = keys.pop().ok_or_else(|| Error::UnexpectedError)?;
                    sibling_keys.push(key);
                }
                // Pop median key - to be added to the parent..
                let median_key = keys.pop().ok_or_else(|| Error::UnexpectedError)?;
                // Populate siblings children.
                for _i in 1..self.b {
                    let child = children.pop().ok_or_else(|| Error::UnexpectedError)?;
                    sibling_children.push(child);
                }
                Ok((
                    median_key,
                    Box::new(Node::new(
                        NodeType::Internal(sibling_children, sibling_keys),
                        false,
                        node.parent_offset.clone(),
                    )),
                ))
            }
            NodeType::Leaf(mut pairs) => {
                let mut sibling_pairs = Vec::<KeyValuePair>::new();
                // Populate siblings pairs.
                for _i in 1..(self.b - 1) {
                    let pair = pairs.pop().ok_or_else(|| Error::UnexpectedError)?;
                    sibling_pairs.push(pair);
                }
                // Pop median key.
                let median_pair = pairs.pop().ok_or_else(|| Error::UnexpectedError)?;
                Ok((
                    Key(median_pair.key),
                    Box::new(Node::new(
                        NodeType::Leaf(sibling_pairs),
                        false,
                        node.parent_offset.clone(),
                    )),
                ))
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// allocate_new_root is a helper method which allocates a new root,
    /// and sets the new roots single child as the old root.
    /// it returns a reference to the old root to be used for calling split_child.
    fn allocate_new_root(&mut self, root: Box<Node>) -> Result<(Box<Node>, Offset), Error> {
        // Keep a copy of the old root
        let mut old_root = root;
        let old_root_offset = self.root_offset.clone();

        // Allocate a new root.
        let Offset(root_offset) = self.root_offset;
        let new_root = Box::new(Node::new(
            NodeType::Internal(vec![Offset(root_offset)], vec![]),
            true,
            None,
        ));
        // write the new root to disk.
        let new_root_offset = self.pager.write_page(Page::try_from(new_root)?)?;
        // Set current root parent and parent_offset to new root.
        old_root.parent_offset = Some(new_root_offset.clone());
        self.pager
            .write_page_at_offset(Page::try_from(old_root.clone())?, Offset(root_offset))?;
        // Set the trees root offset fields.
        self.root_offset = new_root_offset;

        Ok((old_root, old_root_offset))
    }

    /// insert_non_full (recoursively) finds a node rooted at a given non-full node.
    /// to insert a given kv pair.
    fn insert_non_full(
        &mut self,
        mut node: Box<Node>,
        node_offset: Offset,
        kv: KeyValuePair,
    ) -> Result<(), Error> {
        match node.node_type {
            NodeType::Leaf(ref mut pairs) => {
                let idx = pairs.binary_search(&kv).unwrap_or_else(|x| x);
                pairs.insert(idx, kv);
                self.pager
                    .write_page_at_offset(Page::try_from(node)?, node_offset)
            }
            NodeType::Internal(ref children, ref keys) => {
                let idx = keys
                    .binary_search(&Key(kv.key.clone()))
                    .unwrap_or_else(|x| x);
                let child_offset = children.get(idx).ok_or_else(|| Error::UnexpectedError)?;
                let child_page = self.pager.get_page(child_offset.clone())?;
                let child = Box::new(Node::try_from(child_page)?);
                let is_full = self.is_node_full(child.clone())?;
                if is_full {
                    self.split_child(
                        node.clone(),
                        node_offset,
                        child.clone(),
                        child_offset.clone(),
                    )?;
                }
                self.insert_non_full(child, child_offset.clone(), kv)
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// search searches for a specific key in the BTree.
    pub fn search(&mut self, key: String) -> Result<KeyValuePair, Error> {
        let root_page = self.pager.get_page(self.root_offset.clone())?;
        let root = Box::new(Node::try_from(root_page)?);
        self.search_node(root, &key)
    }

    /// search_node recursively searches a sub tree rooted at node for a key
    /// using a Pager to request pages as it traverses the subtree.
    /// if we have traveresed all the way to the leaves and the key was not found the method
    /// returns the leaf node and None indicating the key was not found,
    /// otherwise, continues recursively or return the appropriate error.
    fn search_node(&mut self, node: Box<Node>, search: &str) -> Result<KeyValuePair, Error> {
        match node.node_type {
            NodeType::Internal(children, keys) => {
                for (i, Key(key)) in keys.iter().enumerate() {
                    if search < key {
                        // Retrieve child page from disk and deserialize.
                        let child_offset = children.get(i).ok_or_else(|| Error::UnexpectedError)?;
                        let page = self.pager.get_page(child_offset.clone())?;
                        let child_node = Node::try_from(page)?;
                        return self.search_node(Box::new(child_node), search);
                    }
                }
                Err(Error::KeyNotFound)
            }
            NodeType::Leaf(pairs) => {
                for kv_pair in pairs.iter() {
                    if kv_pair.key.eq(search) {
                        return Ok(kv_pair.clone());
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
