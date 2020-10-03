use crate::node::Node;
use crate::pager::Pager;

/// B+Tree properties.
pub const MAX_BRANCHING_FACTOR: usize = 200;
pub const MIN_BRANCHING_FACTOR: usize = 100;
pub const INTERNAL_NODE_KEYS_LIMIT: usize = MAX_BRANCHING_FACTOR - 1;


/// struct represents an on-disk (persisted) Btree implementation
/// Each node is persisted in itself where the leaf nodes contain the values.
/// This implementation works best when node size are large, say 200+ Kb.
pub struct BTree {
    root: Node,
    pager: Pager,
}

impl BTree {
    fn new(pager: Pager, root: Node) -> BTree {
        BTree{
            pager,
            root
        }
    }
}
