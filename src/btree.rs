use crate::error::Error;
use crate::node::Node;
use std::path::Path;
use uuid::Uuid;

/// struct represents an on-disk (persisted) Btree implementation
/// Each node is persisted in itself where the leaf nodes contain the values.
/// This implementation works best when node size are large, say 200+ Kb.
pub struct BTree<'a, T>
where
    T: Ord,
{
    // b - 1 <= #keys per node <= 2b - 1
    b: usize,
    root: &'a mut Node<'a, T>,
}

impl<'a, T> BTree<'a, T>
where
    T: Ord,
{
    fn new(root: &'a mut Node<'a, T>, b: usize) -> BTree<'a, T> {
        BTree { root, b }
    }

    /// recoursively searches for a key in the tree.
    fn search(&'a self, search_key: T) -> Option<&T> {
        self.search_key(search_key, self.root)
    }

    /// search_key performs a linear exhaustive search for a key ('search_key')
    /// in the vector of keys at a given node.
    /// If a match is found it returns it, otherwise, it continues recoursively
    /// or terminates in case of a leaf node.
    fn search_key(&self, search_key: T, node: &'a Node<'a, T>) -> Option<&'a T> {
        for i in 0..node.keys.len() - 1 {
            let key = match node.keys.get(i) {
                Some(key) => key,
                None => return None,
            };
            if search_key == **key {
                return Some(key);
            }
            if search_key < **key {
                let child = match node.children.get(i) {
                    None => return None,
                    Some(child) => child,
                };
                return self.search_key(search_key, child);
            }
            // key isn't found in current node set of keys
            if i == node.keys.len() - 1 {
                if node.leaf {
                    return None;
                }
                // search in right most child.
                let child = match node.children.get(node.children.len()) {
                    None => return None,
                    Some(child) => child,
                };
                return self.search_key(search_key, child);
            }
        }
        None
    }

    /// insert inserts a new key to the tree.
    fn insert(&'a mut self, key: T) -> Result<(), Error> {
        Ok(())
    }

    /// splits node into two.
    /// node is the node being split.
    /// parent is the parent of the given node.
    /// idx is the index of the node being split.
    /// For example, consider the following B-Tree with b=2:
    ///       [ D ]
    ///      /     \
    ///  [A,B]   [E,F,G]
    /// If we were to insert a new key (say H) to the tree we would need to split the right leaf.
    ///       [D ,F]
    ///     /   /   \
    ///  [A,B] [E]  [G]   
    fn split_node(
        &'a mut self,
        node: &'a mut Node<'a, T>,
        parent: &'a mut Node<'a, T>,
        idx: usize,
    ) -> Result<(), Error> {
        // alocate a new node sibling, node which contains the t-1 last keys.
        // as well as the t+1 children (in case of an internal node.)
        let mut sibling = Node::<T>::new(
            Vec::<&'a T>::new(),
            Vec::<Node<'a, T>>::new(),
            node.parent,
            node.leaf,
        );
        // copy keys over to the newly allocated sibling.
        // copies over b-1 keys from b to 2b-2.
        for i in 1..(self.b - 1) {
            let key_to_copy = node.keys.remove(self.b);
            sibling.keys.push(key_to_copy)
        }
        // copy child ptrs over to the newly allocated sibling.
        // moves over b ptrs from b .. 2b - 1
        if !node.leaf {
            for i in 1..self.b {
                let ptr_to_copy = node.children.remove(self.b);
                sibling.children.push(ptr_to_copy);
            }
        }
        // insert sibling as a child of the parent.
        parent.children.insert(idx + 1, sibling);
        // insert median key of the node being split into parent.
        let median = node.keys.remove(self.b - 1);
        parent.keys.insert(idx, median);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::btree::BTree;
    use crate::node::Node;

    #[test]
    fn test_insert() {
        let mut root = Node::<char>::new(
            Vec::<&char>::new(),
            Vec::<Node<char>>::new(),
            None,
            true,
        );
        let tree = BTree::<char>::new(&mut root, 7);
    }
}
