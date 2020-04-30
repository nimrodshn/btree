pub struct Node<'a, T> where T: Ord {
    pub keys : Vec<T>,
    pub children : Vec<&'a Node<'a, T>>,
    pub leaf: bool
}

impl <'a, T> Node<'a,T> where T: Ord {
    fn new(keys: Vec<T>, children: Vec<&'a Node<'a, T>>, leaf: bool) -> Node<T> {
        Node {
            keys,
            children,
            leaf
        }
    }
} 