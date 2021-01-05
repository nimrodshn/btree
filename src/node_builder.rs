use crate::node::{Node};
use crate::page::Page;
use crate::error::Error;

pub trait NodeBuilder {
    fn build(&self) -> Result<Node, Error>;
}

pub struct InternalNodeBuilder {
    pub parent_pointer: usize,
    pub is_root: bool,
    pub children: Vec<usize>,
    pub keys: Vec<usize>
}

impl NodeBuilder for InternalNodeBuilder { 
    fn build(&self) -> Result<Node,Error> {
        let mut result: [u8; PAGE_SIZE] = [0x00; PAGE_SIZE];
        // Common node header data
        result[IS_ROOT_OFFSET] = node.is_root.to_byte();
        result[NODE_TYPE_OFFSET] = node.node_type.into();
        result[PARENT_POINTER_OFFSET..PARENT_POINTER_OFFSET + PARENT_POINTER_SIZE]
            .clone_from_slice(&node.parent_offset.to_be_bytes());
        
        Ok(Page::new(result))
    }
}

impl InternalNodeBuilder {

}
