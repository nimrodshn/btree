use crate::error::Error;
use crate::node::Node;
use crate::node_type::{Key, KeyValuePair, NodeType, Offset};
use crate::page::Page;
use crate::pager::Pager;
use std::cmp;
use std::convert::TryFrom;
use std::path::Path;

/// B+Tree properties.
pub const MAX_BRANCHING_FACTOR: usize = 200;
pub const NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;

/// BTree struct represents an on-disk B+tree.
/// Each node is persisted in the table file, the leaf nodes contain the values.
pub struct BTree {
    pager: Pager,
    b: usize,
    root_offset: Offset,
}

/// BtreeBuilder is a Builder for the BTree struct.
pub struct BTreeBuilder {
    /// Path to the tree file.
    path: &'static Path,
    /// The BTree parameter, an inner node contains no more than 2*b-1 keys and no less than b-1 keys
    /// and no more than 2*b children and no less than b children.
    b: usize,
}

impl BTreeBuilder {
    pub fn new() -> BTreeBuilder {
        BTreeBuilder {
            path: Path::new(""),
            b: 0,
        }
    }

    pub fn path(mut self, path: &'static Path) -> BTreeBuilder {
        self.path = path;
        self
    }

    pub fn b_parameter(mut self, b: usize) -> BTreeBuilder {
        self.b = b;
        self
    }

    pub fn build(&self) -> Result<BTree, Error> {
        if self.path.to_string_lossy() == "" {
            return Err(Error::UnexpectedError);
        }
        if self.b == 0 {
            return Err(Error::UnexpectedError);
        }

        let mut pager = Pager::new(&self.path)?;
        let root = Node::new(NodeType::Leaf(vec![]), true, None);
        let root_offset = pager.write_page(Page::try_from(&root)?)?;
        Ok(BTree {
            pager,
            b: self.b,
            root_offset,
        })
    }
}

impl Default for BTreeBuilder {
    // A default BTreeBuilder provides a builder with:
    /// - b parameter set to 200
    /// - path set to '/tmp/db'.
    fn default() -> Self {
        BTreeBuilder::new()
            .b_parameter(200)
            .path(Path::new("/tmp/db"))
    }
}

impl BTree {
    fn is_node_full(&self, node: &Node) -> Result<bool, Error> {
        match &node.node_type {
            NodeType::Leaf(pairs) => Ok(pairs.len() == (2 * self.b - 1)),
            NodeType::Internal(_, keys) => Ok(keys.len() == (2 * self.b - 1)),
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    fn is_node_underflow(&self, node: &Node) -> Result<bool, Error> {
        match &node.node_type {
            // A root cannot really be "underflowing" as it can contain less than b-1 keys / pointers.
            NodeType::Leaf(pairs) => Ok(pairs.len() < self.b - 1 && !node.is_root),
            NodeType::Internal(_, keys) => Ok(keys.len() < self.b - 1 && !node.is_root),
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
            let mut new_root = Node::new(NodeType::Internal(vec![], vec![]), true, None);
            // write the new root to disk.
            let new_root_offset = self.pager.write_page(Page::try_from(&new_root)?)?;
            // Set the current roots parent to the new root.
            old_root.parent_offset = Some(new_root_offset.clone());
            old_root.is_root = false;
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
                NodeType::Internal(vec![old_root_offset, sibling_offset], vec![median]);
            // Write the new_root to disk.
            self.pager
                .write_page_at_offset(Page::try_from(&new_root)?, &self.root_offset)?;
            // Assign the new root.
            root = new_root;
        }
        self.insert_non_full(&mut root, self.root_offset.clone(), kv)
    }

    /// insert_non_full (recursively) finds a node rooted at a given non-full node.
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
                    // split will split the child at b leaving the [0, b-1] keys
                    // while moving the set of [b, 2b-1] keys to the sibling.
                    let (median, mut sibling) = child.split(self.b)?;
                    self.pager
                        .write_page_at_offset(Page::try_from(&child)?, &child_offset)?;
                    // Write the newly created sibling to disk.
                    let sibling_offset = self.pager.write_page(Page::try_from(&sibling)?)?;
                    // Siblings keys are larger than the splitted child thus need to be inserted
                    // at the next index.
                    children.insert(idx + 1, sibling_offset.clone());
                    keys.insert(idx, median.clone());

                    // Write the parent page to disk.
                    self.pager
                        .write_page_at_offset(Page::try_from(&*node)?, &node_offset)?;
                    // Continue recursively.
                    if kv.key <= median.0 {
                        self.insert_non_full(&mut child, child_offset, kv)
                    } else {
                        self.insert_non_full(&mut sibling, sibling_offset, kv)
                    }
                } else {
                    self.insert_non_full(&mut child, child_offset, kv)
                }
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

    /// search_node recursively searches a sub tree rooted at node for a key.
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

    /// delete deletes a given key from the tree.
    pub fn delete(&mut self, key: Key) -> Result<(), Error> {
        self.delete_key_from_subtree(key, &self.root_offset.clone())
    }

    /// delete key from subtree recursively traverses a tree rooted at a node in certain offset
    /// until it finds the given key and delets.
    fn delete_key_from_subtree(&mut self, key: Key, offset: &Offset) -> Result<(), Error> {
        let page = self.pager.get_page(offset)?;
        let mut node = Node::try_from(page)?;
        match &mut node.node_type {
            NodeType::Leaf(ref mut pairs) => {
                let key_idx = pairs
                    .binary_search_by_key(&key, |kv| Key(kv.key.clone()))
                    .map_err(|_| Error::KeyNotFound)?;
                pairs.remove(key_idx);
                self.pager
                    .write_page_at_offset(Page::try_from(&node)?, offset)?;
                // Check for underflow - if it occures,
                // we need to merge with a sibling.
                // this can only occur if node is not the root (as it cannot "underflow").
                // continue recoursively up the tree.
                self.borrow_if_needed(node, &key)?;
            }
            NodeType::Internal(children, keys) => {
                let node_idx = keys.binary_search(&key).unwrap_or_else(|x| x);
                // Retrieve child page from disk and deserialize.
                // And continue recoursively.
                let child_offset = children.get(node_idx).ok_or(Error::UnexpectedError)?;
                return self.delete_key_from_subtree(key, child_offset);
            }
            NodeType::Unexpected => return Err(Error::UnexpectedError),
        }
        Ok(())
    }

    /// borrow_if_needed checks the node for underflow (following a removal of a key),
    /// if it underflows it is merged with a sibling node, and than called recoursively
    /// up the tree.
    fn borrow_if_needed(&mut self, node: Node, key: &Key) -> Result<(), Error> {
        if self.is_node_underflow(&node)? {
            // Fetch the sibling from the parent -
            // This could be quicker if we implement sibling pointers.
            let parent_offset = node.parent_offset.clone().ok_or(Error::UnexpectedError)?;
            let parent_page = self.pager.get_page(&parent_offset)?;
            let mut parent_node = Node::try_from(parent_page)?;
            // The parent has to be an "internal" node.
            match parent_node.node_type {
                NodeType::Internal(ref mut children, ref keys) => {
                    let idx = keys.binary_search(&key).unwrap_or_else(|x| x);
                    // The sibling is in idx +- 1 as the above index led
                    // the downward search to node.
                    let sibling_idx;
                    match idx > 0 {
                        false => sibling_idx = idx + 1,
                        true => sibling_idx = idx - 1,
                    }

                    let sibling_offset = children.get(sibling_idx).ok_or(Error::UnexpectedError)?;
                    let sibling_page = self.pager.get_page(sibling_offset)?;
                    let sibling = Node::try_from(sibling_page)?;
                    let merged_node = self.merge(node, sibling)?;
                    let merged_node_offset =
                        self.pager.write_page(Page::try_from(&merged_node)?)?;
                    // remove the old nodes.
                    children.remove(idx);
                    children.remove(sibling_idx);
                    // write the new node in place.
                    let merged_node_idx = cmp::min(idx, sibling_idx);
                    children.insert(merged_node_idx, merged_node_offset);
                    // write the updated parent back to disk and continue up the tree.
                    self.pager
                        .write_page_at_offset(Page::try_from(&parent_node)?, &parent_offset)?;
                    return self.borrow_if_needed(parent_node, &key);
                }
                _ => return Err(Error::UnexpectedError),
            }
        }
        Ok(())
    }

    // merges two *sibling* nodes, it assumes the following:
    // 1. the two nodes are of the same type.
    // 2. the two nodes do not accumulate to an overflow,
    // i.e. |first.keys| + |second.keys| <= [2*(b-1) for keys or 2*b for offsets].
    fn merge(&self, first: Node, second: Node) -> Result<Node, Error> {
        match first.node_type {
            NodeType::Leaf(first_pairs) => {
                if let NodeType::Leaf(second_pairs) = second.node_type {
                    let merged_pairs: Vec<KeyValuePair> = first_pairs
                        .into_iter()
                        .chain(second_pairs.into_iter())
                        .collect();
                    let node_type = NodeType::Leaf(merged_pairs);
                    Ok(Node::new(node_type, first.is_root, first.parent_offset))
                } else {
                    Err(Error::UnexpectedError)
                }
            }
            NodeType::Internal(first_offsets, first_keys) => {
                if let NodeType::Internal(second_offsets, second_keys) = second.node_type {
                    let merged_keys: Vec<Key> = first_keys
                        .into_iter()
                        .chain(second_keys.into_iter())
                        .collect();
                    let merged_offsets: Vec<Offset> = first_offsets
                        .into_iter()
                        .chain(second_offsets.into_iter())
                        .collect();
                    let node_type = NodeType::Internal(merged_offsets, merged_keys);
                    Ok(Node::new(node_type, first.is_root, first.parent_offset))
                } else {
                    Err(Error::UnexpectedError)
                }
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// print_sub_tree is a helper function for recursively printing the nodes rooted at a node given by its offset.
    fn print_sub_tree(&mut self, prefix: String, offset: Offset) -> Result<(), Error> {
        println!("{}Node at offset: {}", prefix, offset.0);
        let curr_prefix = format!("{}|->", prefix);
        let page = self.pager.get_page(&offset)?;
        let node = Node::try_from(page)?;
        match node.node_type {
            NodeType::Internal(children, keys) => {
                println!("{}Keys: {:?}", curr_prefix, keys);
                println!("{}Children: {:?}", curr_prefix, children);
                let child_prefix = format!("{}   |  ", prefix);
                for child_offset in children {
                    self.print_sub_tree(child_prefix.clone(), child_offset)?;
                }
                Ok(())
            }
            NodeType::Leaf(pairs) => {
                println!("{}Key value pairs: {:?}", curr_prefix, pairs);
                Ok(())
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }

    /// print is a helper for recursively printing the tree.
    pub fn print(&mut self) -> Result<(), Error> {
        println!();
        self.print_sub_tree("".to_string(), self.root_offset.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::error::Error;

    #[test]
    fn search_works() -> Result<(), Error> {
        use crate::btree::BTreeBuilder;
        use crate::node_type::KeyValuePair;
        use std::path::Path;

        let mut btree = BTreeBuilder::new()
            .path(Path::new("/tmp/db"))
            .b_parameter(2)
            .build()?;
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
        use crate::btree::BTreeBuilder;
        use crate::node_type::KeyValuePair;
        use std::path::Path;

        let mut btree = BTreeBuilder::new()
            .path(Path::new("/tmp/db"))
            .b_parameter(2)
            .build()?;
        btree.insert(KeyValuePair::new("a".to_string(), "shalom".to_string()))?;
        btree.insert(KeyValuePair::new("b".to_string(), "hello".to_string()))?;
        btree.insert(KeyValuePair::new("c".to_string(), "marhaba".to_string()))?;
        btree.insert(KeyValuePair::new("d".to_string(), "olah".to_string()))?;
        btree.insert(KeyValuePair::new("e".to_string(), "salam".to_string()))?;
        btree.insert(KeyValuePair::new("f".to_string(), "hallo".to_string()))?;
        btree.insert(KeyValuePair::new("g".to_string(), "Konnichiwa".to_string()))?;
        btree.insert(KeyValuePair::new("h".to_string(), "Ni hao".to_string()))?;
        btree.insert(KeyValuePair::new("i".to_string(), "Ciao".to_string()))?;

        let mut kv = btree.search("a".to_string())?;
        assert_eq!(kv.key, "a");
        assert_eq!(kv.value, "shalom");

        kv = btree.search("b".to_string())?;
        assert_eq!(kv.key, "b");
        assert_eq!(kv.value, "hello");

        kv = btree.search("c".to_string())?;
        assert_eq!(kv.key, "c");
        assert_eq!(kv.value, "marhaba");

        kv = btree.search("d".to_string())?;
        assert_eq!(kv.key, "d");
        assert_eq!(kv.value, "olah");

        kv = btree.search("e".to_string())?;
        assert_eq!(kv.key, "e");
        assert_eq!(kv.value, "salam");

        kv = btree.search("f".to_string())?;
        assert_eq!(kv.key, "f");
        assert_eq!(kv.value, "hallo");

        kv = btree.search("g".to_string())?;
        assert_eq!(kv.key, "g");
        assert_eq!(kv.value, "Konnichiwa");

        kv = btree.search("h".to_string())?;
        assert_eq!(kv.key, "h");
        assert_eq!(kv.value, "Ni hao");

        kv = btree.search("i".to_string())?;
        assert_eq!(kv.key, "i");
        assert_eq!(kv.value, "Ciao");

        Ok(())
    }

    #[test]
    fn delete_works() -> Result<(), Error> {
        use crate::btree::BTreeBuilder;
        use crate::error::Error;
        use crate::node_type::{Key, KeyValuePair};
        use std::path::Path;

        let mut btree = BTreeBuilder::new()
            .path(Path::new("/tmp/db"))
            .b_parameter(2)
            .build()?;
        btree.insert(KeyValuePair::new("d".to_string(), "olah".to_string()))?;
        btree.insert(KeyValuePair::new("e".to_string(), "salam".to_string()))?;
        btree.insert(KeyValuePair::new("f".to_string(), "hallo".to_string()))?;
        btree.insert(KeyValuePair::new("a".to_string(), "shalom".to_string()))?;
        btree.insert(KeyValuePair::new("b".to_string(), "hello".to_string()))?;
        btree.insert(KeyValuePair::new("c".to_string(), "marhaba".to_string()))?;

        let kv = btree.search("c".to_string())?;
        assert_eq!(kv.key, "c");
        assert_eq!(kv.value, "marhaba");

        btree.delete(Key("c".to_string()))?;

        let res = btree.search("c".to_string());
        assert!(matches!(res, Err(Error::KeyNotFound)));

        Ok(())
    }
}
