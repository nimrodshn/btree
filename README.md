# btree

[![Build status](https://github.com/nimrodshn/btree/actions/workflows/build.yml/badge.svg)](https://github.com/nimrodshn/btree/actions)

A **persistent** B+Tree implementation, designed as an index for a key value store, inspired by [SQLite](https://www.sqlite.org/index.html).

## Design
Each `BTree` struct is associated with a file that contains its nodes. Each node has a predefined structure.

Unit tests serve as helpful examples of API usage.

## On disk nodes structure
There are two `NodeType` variants - `Internal` and `Leaf`; Each variant has its own predefined structure on disk.
A leaf node has the following structure:
```
| IS-ROOT 1-byte| NODE-TYPE 1-byte | PARENT OFFSET - 8 bytes | Number of pairs - 8 bytes |
| Key #0 - 10 bytes | Value #0 - 10 bytes | ...
| Key #N - 10 bytes | Value #N - 10 bytes |
```

While the structure of an internal node on disk is the following:
```
| IS-ROOT 1-byte | NODE-TYPE 1-byte | PARENT OFFSET - 8 bytes | Number of children - 8 bytes |
| Key #0 - 10 bytes | Key #2 - 10 bytes | ...
| Child Offset #0 - 8 bytes | Child offset #1 - 8 bytes | ...
```

## API

### From disk to memory and back
Nodes are mapped to pages on disk with `TryFrom` methods implemented for easier de/serialization of nodes to pages and back.

```
let some_leaf = Node::new(
   NodeType::Leaf(vec![
         KeyValuePair::new("foo".to_string(), "bar".to_string()),
         KeyValuePair::new("lebron".to_string(), "james".to_string()),
         KeyValuePair::new("ariana".to_string(), "grande".to_string()),
   ]),
   true,
   None,
);

// Serialize data.
let page = Page::try_from(&some_leaf)?;
// Deserialize back the page.
let res = Node::try_from(page)?;
```

See tests at `src/page.rs` and `src/node.rs` for more information.

### Writing and Reading key-value pairs.
```
// Initialize a new BTree;
// The BTree nodes are stored in file '/tmp/db' (created if does not exist)
// with parameter b=2.
 let mut btree = BTreeBuilder::new()
            .path(Path::new("/tmp/db"))
            .b_parameter(2)
            .build()?;

// Write some data.
btree.insert(KeyValuePair::new("a".to_string(), "shalom".to_string()))?;
btree.insert(KeyValuePair::new("b".to_string(), "hello".to_string()))?;
btree.insert(KeyValuePair::new("c".to_string(), "marhaba".to_string()))?;

// Read it back.
let mut kv = btree.search("b".to_string())?;
assert_eq!(kv.key, "b");
assert_eq!(kv.value, "hello");

kv = btree.search("c".to_string())?;
assert_eq!(kv.key, "c");
assert_eq!(kv.value, "marhaba");
```

### Deleting key-value pairs.
```
// Initialize a new BTree.
let mut btree = BTreeBuilder::new()
      .path(Path::new("/tmp/db"))
      .b_parameter(2)
      .build()?;

// Write some data.
btree.insert(KeyValuePair::new("d".to_string(), "olah".to_string()))?;
btree.insert(KeyValuePair::new("e".to_string(), "salam".to_string()))?;
btree.insert(KeyValuePair::new("f".to_string(), "hallo".to_string()))?;
btree.insert(KeyValuePair::new("a".to_string(), "shalom".to_string()))?;
btree.insert(KeyValuePair::new("b".to_string(), "hello".to_string()))?;
btree.insert(KeyValuePair::new("c".to_string(), "marhaba".to_string()))?;

// Find the key.
let kv = btree.search("c".to_string())?;
assert_eq!(kv.key, "c");
assert_eq!(kv.value, "marhaba");

// Delete the key.
btree.delete(Key("c".to_string()))?;

// Sanity check.
let res = btree.search("c".to_string());
assert!(matches!(
      res,
      Err(Error::KeyNotFound)
));
```

## License
MIT.
