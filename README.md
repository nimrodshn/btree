# btree

A **persistent** B+Tree implementation, optimized to be used as a [Sorted String Table](https://stackoverflow.com/questions/2576012/what-is-an-sstable).

This project is ment to support the most simple common use case - a key-value index.

## Design
This project **only** supports a single index, that is, a sparse [Sorted String Table](https://stackoverflow.com/questions/2576012/what-is-an-sstable).

Each `BTree` Object is associated with a file that contains its data. The `BTree` root is always at offset zero. Each node is persisted in the associated file and has a predefined structure.

## License
MIT.