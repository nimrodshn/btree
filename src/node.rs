pub struct Node<'a, T> where T: Ord {
    pub keys : Vec<T>,
    pub pointers : Vec<&'a Node<'a, T>>,
    pub leaf: bool
}