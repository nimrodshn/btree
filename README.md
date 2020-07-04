# btree

A **persistent** B+Tree implementation, designed to be used as a [Sorted String Table](https://stackoverflow.com/questions/2576012/what-is-an-sstable).

This project is ment to support the most simple common use case - a key-value index.

## Design
This project **only** supports a single index, that is, a sparse [Sorted String Table](https://stackoverflow.com/questions/2576012/what-is-an-sstable).

Each `BTree` struct is associated with a file that contains its nodes. The `BTree` root is always at offset zero. Each node is persisted in the associated file and has a predefined structure.

## Node structure - a simple example
There are two `NodeType` variants: `Internal` and `Leaf` (see `src/node.rs`). Each variant has its own defined structure on disk, for example, the `Leaf` type might look like:
```
   0x01,                                           // Is-Root byte.
   0x02,                                           // Node type byte.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // Key size.
   0x68, 0x65, 0x6c, 0x6c, 0x6f,                   // "hello"
   0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x05, // Value size.
   0x77, 0x6f, 0x72, 0x6c, 0x64,                   // "world"
```

## License
MIT.