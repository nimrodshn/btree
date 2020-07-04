use crate::error::Error;
use crate::node::Node;
use crate::pager::Pager;
use std::path::Path;
use uuid::Uuid;

/// struct represents an on-disk (persisted) Btree implementation
/// Each node is persisted in itself where the leaf nodes contain the values.
/// This implementation works best when node size are large, say 200+ Kb.
pub struct BTree {
    // b - 1 <= #keys per node <= 2b - 1
    // b <= #children per node <= 2b
    b: usize,
    root: Node,
    pager: Pager,
}
