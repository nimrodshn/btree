/// A single page size.
/// Each page represents a node in the BTree.
const PAGE_SIZE: u32 = 4096;

/// Common Node header layout.
const IS_ROOT_SIZE: u32 = 1;
const IS_ROOT_OFFSET: u32 = 0;
const PARENT_POINTER_OFFSET: u32 = 1;
const PARENT_POINTER_SIZE: u32 = 4;
