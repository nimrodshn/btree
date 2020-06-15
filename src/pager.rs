use std::fs::File;
use std::collections::HashMap;
use crate::node::Node;

pub struct Pager{
    fd: File,
    num_pages: u32,
    cache: HashMap<String, Node>,
}