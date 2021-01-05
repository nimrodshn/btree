![](img/btree.jpg)
# btree

**IMPORTANT** 

This project is ongoing and considered work-in-progress - contributions are welcome :).

A **persistent** B+Tree implementation, designed as an index for a key value store, inspired by [SQLite](https://www.sqlite.org/index.html).

## Design
This project **only** supports a single index per BTree although multiple trees can be used as multiple indexes in a table.

Each `BTree` struct is associated with a file that contains its nodes (see `src/pager.rs`). The `BTree` root is always at offset zero. Each node is persisted in the associated file and has a predefined structure.

## A Leaf Node structure - a simple example
There are two `NodeType` variants: `Internal` and `Leaf` (see `src/node.rs`). Each variant has its own defined structure on disk, for example, the `Leaf` type might look like:
```
   0x01,                                           // Is-Root byte.
   0x02,                                           // Node type byte.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
   0x68, 0x65, 0x6c, 0x6c, 0x6f,                   // "hello"
   0x77, 0x6f, 0x72, 0x6c, 0x64,                   // "world"
```

Both Key and Value are both persisted in the tress nodes; values are persisted in the leaf nodes to avoid an extra disk access for values.

As a consequence the values and keys are limited in size to 10 Bytes long.

## An Internal Node structure - a simple example
Here is a simple example of an "internal" node:
```
   0x01, // Is-Root byte.
   0x01, // Node type byte.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, // Number of children.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, // 4096  (2nd Page)
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, // 8192  (3rd Page)
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, // 12288 (4th Page)
   0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
   0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
```
Note that the number of keys in an internal node is always one less than the number of child pointers.

For more information about the on-disk data strcuture see the tests at `src/nodes.rs`.

## License
MIT.
