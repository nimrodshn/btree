use std::io::Error;

pub trait Persist {
    fn write_to_file() -> Result<u32, Error>;
}

pub struct BTree<'a, T> where T: Ord {
    branching_factor : u32,
    root : &'a mut Node<'a, T>,
}

pub struct Node<'a, T> where T: Ord {
    keys : Vec<T>,
    pointers : Vec<&'a Node<'a, T>>
}

impl <'a, T> BTree<'a, T> where T: Ord  {
    fn new(root: &'a mut Node<'a,T>, branching_factor: u32) -> BTree<'a, T> {
        BTree{
            root,
            branching_factor
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
