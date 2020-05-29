use std::error::Error;
use std::option::Option;

pub struct Node<'a, T>
where
    T: Ord,
{
    pub keys: Vec<&'a T>,
    pub children: Vec<Node<'a, T>>,
    pub parent: Option<&'a Node<'a, T>>,
    pub leaf: bool,
}

impl<'a, T> Node<'a, T>
where
    T: Ord,
{
    pub fn new(
        keys: Vec<&'a T>,
        children: Vec<Node<'a, T>>,
        parent: Option<&'a Node<'a,T>>,
        leaf: bool,
    ) -> Node<'a, T> {
        Node {
            keys,
            children,
            parent,
            leaf,
        }
    }
}
