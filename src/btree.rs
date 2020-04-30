use crate::node::Node;
use crate::error::Error;
use std::io;

pub trait Persist {
    fn write_to_file() -> Result<u32, io::Error>;
}

pub struct BTree<'a, T> where T: Ord {
    // ( node_capacity / 2 ) <= # keys a node can contain <= node_capacity
    node_capacity : usize,
    root : &'a mut Node<'a, T>,
}

impl <'a, T> BTree<'a, T> where T: Ord  {
    fn new(root: &'a mut Node<'a,T>, node_capacity: usize) -> BTree<'a, T> {
        BTree{
            root,
            node_capacity
        }
    }

    // recoutsively searches for a key in the tree.
    fn search(&'a self, search_key: T) -> Option<&T> {
        self.search_key(search_key, self.root)
    }

    // search_key performs a linear exhaustive search of the parameters 'search_key'
    // in the vector of keys at a given node.
    // If a match is found it returns it, otherwise, it continues recoursively
    // or terminates in case of a leaf node.
    fn search_key(&self, search_key: T, node: &'a Node<'a, T>) -> Option<&'a T> {
        for  i in 0..node.keys.len()-1 {
            let key = match node.keys.get(i) {
                Some(key) => key,
                None => return None,
            };
            if search_key == *key {
                return Some(key)
            }
            if search_key < *key {
                let child = match node.children.get(i) {
                    None => return None,
                    Some(child) => child,
                };
                return self.search_key(search_key, child)
            }
            // key isn't found in current node set of keys
            if  i == node.keys.len()-1 {
                if node.leaf {
                    return None
                }
                // search in right most child.
                let child = match node.children.get(node.children.len()) {
                    None => return None,
                    Some(child) => child,
                };
                return self.search_key(search_key, child)
            }
        }
        None
    }

    // insert inserts a new key to the tree.
    fn insert(&'a mut self, key: T) -> Result<(), Error> {
        BTree::search_node_for_insert(key, self.root, self.node_capacity) 
    }

    // search_node_for_insert finds traverses the tree to find a node to house a given key.
    fn search_node_for_insert(search_key: T, node: &'a mut Node<'a, T>, node_capacity: usize) -> Result<(), Error> {
        for  i in 0..node.keys.len()-1 {
            let key = match node.keys.get(i) {
                Some(key) => key,
                None => return Err(Error::KeyNotFound),
            };
            if search_key < *key {
                if node.keys.len() < node_capacity {
                    node.keys.insert(i, search_key);
                } else {
                    // TODO: Split node.
                }
                return Ok(())
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
