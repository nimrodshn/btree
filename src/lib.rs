pub struct BTree<'a> {
    branching_factor : u32,
    root : &'a mut Node<'a>,
}

pub struct Node<'a> {
    keys : Vec<&'a [u8]>,
    pointers : Vec<Node<'a>>
}

impl <'a> BTree<'a> {
    fn new(root: &'a mut Node<'a>, branching_factor: u32) -> BTree<'a> {
        BTree{
            root,
            branching_factor
        }
    }

    // TODO: implement me.
    fn insert(&mut self, key: &[u8]) {}
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
