use crate::node::Node;
use std::io::Error;

pub trait Persist {
    fn write_to_file() -> Result<u32, Error>;
}

pub struct BTree<'a, T> where T: Ord {
    branching_factor : u32,
    root : &'a mut Node<'a, T>,
}

impl <'a, T> BTree<'a, T> where T: Ord  {
    fn new(root: &'a mut Node<'a,T>, branching_factor: u32) -> BTree<'a, T> {
        BTree{
            root,
            branching_factor
        }
    }

    // recoutsively searches for a node in the tree.
    fn search(&'a self, search_key: T) -> Option<&T> {
        self.search_node(search_key, self.root)
    }

    // search_node performs a linear exhaustive search of the parameters 'search_key'
    // in the vector of keys at a given node.
    // If a match is found it returns it, continues recoursively
    // or terminates in case of a leaf node.
    fn search_node(&self, search_key: T, node: &'a Node<'a, T>) -> Option<&'a T> {
        for  i in 0..node.keys.len()-1 {
            let key = match node.keys.get(i) {
                Some(key) => key,
                None => return None,
            };
            if search_key == *key {
                return Some(key)
            }
            if search_key < *key {
                let child = match node.pointers.get(i) {
                    None => return None,
                    Some(child) => child,
                };
                return self.search_node(search_key, child)
            }
            // key isn't found in current node set of keys
            if  i == node.keys.len()-1 {
                if node.leaf {
                    return None
                }
                // search in right most child.
                let child = match node.pointers.get(node.pointers.len()) {
                    None => return None,
                    Some(child) => child,
                };
                return self.search_node(search_key, child)
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
