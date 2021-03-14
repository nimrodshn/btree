use crate::error::Error;
use crate::node_type::{Key, KeyValuePair, NodeType, Offset};
use crate::page::Page;
use crate::page_layout::{
    FromByte, INTERNAL_NODE_HEADER_SIZE, INTERNAL_NODE_NUM_CHILDREN_OFFSET, IS_ROOT_OFFSET,
    KEY_SIZE, LEAF_NODE_HEADER_SIZE, LEAF_NODE_NUM_PAIRS_OFFSET, NODE_TYPE_OFFSET,
    PARENT_POINTER_OFFSET, PTR_SIZE, VALUE_SIZE,
};
use std::convert::TryFrom;
use std::str;

/// Node represents a node in the BTree occupied by a single page in memory.
#[derive(Clone, Debug)]
pub struct Node {
    pub node_type: NodeType,
    pub is_root: bool,
    pub parent_offset: Option<Offset>,
}

// Node represents a node in the B-Tree.
impl Node {
    pub fn new(node_type: NodeType, is_root: bool, parent_offset: Option<Offset>) -> Node {
        Node {
            node_type,
            is_root,
            parent_offset,
        }
    }

    /// split creates a sibling node from a given node by splitting the node in two around a median.
    pub fn split(&mut self, b: usize) -> Result<(Key, Node), Error> {
        match self.node_type {
            NodeType::Internal(ref mut children, ref mut keys) => {
                // Populate siblings keys.
                let mut sibling_keys = keys.split_off(b - 1);
                // Pop median key - to be added to the parent..
                let median_key = sibling_keys.remove(0);
                // Populate siblings children.
                let sibling_children = children.split_off(b);
                Ok((
                    median_key,
                    Node::new(
                        NodeType::Internal(sibling_children, sibling_keys),
                        false,
                        self.parent_offset.clone(),
                    ),
                ))
            }
            NodeType::Leaf(ref mut pairs) => {
                // Populate siblings pairs.
                let sibling_pairs = pairs.split_off(b);
                // Pop median key.
                let median_pair = pairs.get(b - 1).ok_or(Error::UnexpectedError)?.clone();

                Ok((
                    Key(median_pair.key),
                    Node::new(
                        NodeType::Leaf(sibling_pairs),
                        false,
                        self.parent_offset.clone(),
                    ),
                ))
            }
            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }
}

/// Implement TryFrom<Page> for Node allowing for easier
/// deserialization of data from a Page.
impl TryFrom<Page> for Node {
    type Error = Error;
    fn try_from(page: Page) -> Result<Node, Error> {
        let raw = page.get_data();
        let node_type = NodeType::from(raw[NODE_TYPE_OFFSET]);
        let is_root = raw[IS_ROOT_OFFSET].from_byte();
        let parent_offset: Option<Offset>;
        if is_root {
            parent_offset = None;
        } else {
            parent_offset = Some(Offset(page.get_value_from_offset(PARENT_POINTER_OFFSET)?));
        }

        match node_type {
            NodeType::Internal(mut children, mut keys) => {
                let num_children = page.get_value_from_offset(INTERNAL_NODE_NUM_CHILDREN_OFFSET)?;
                let mut offset = INTERNAL_NODE_HEADER_SIZE;
                for _i in 1..=num_children {
                    let child_offset = page.get_value_from_offset(offset)?;
                    children.push(Offset(child_offset));
                    offset += PTR_SIZE;
                }

                // Number of keys is always one less than the number of children (i.e. branching factor)
                for _i in 1..num_children {
                    let key_raw = page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += KEY_SIZE;
                    // Trim leading or trailing zeros.
                    keys.push(Key(key.trim_matches(char::from(0)).to_string()));
                }
                Ok(Node::new(
                    NodeType::Internal(children, keys),
                    is_root,
                    parent_offset,
                ))
            }

            NodeType::Leaf(mut pairs) => {
                let mut offset = LEAF_NODE_NUM_PAIRS_OFFSET;
                let num_keys_val_pairs = page.get_value_from_offset(offset)?;
                offset = LEAF_NODE_HEADER_SIZE;

                for _i in 0..num_keys_val_pairs {
                    let key_raw = page.get_ptr_from_offset(offset, KEY_SIZE);
                    let key = match str::from_utf8(key_raw) {
                        Ok(key) => key,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += KEY_SIZE;

                    let value_raw = page.get_ptr_from_offset(offset, VALUE_SIZE);
                    let value = match str::from_utf8(value_raw) {
                        Ok(val) => val,
                        Err(_) => return Err(Error::UTF8Error),
                    };
                    offset += VALUE_SIZE;

                    // Trim leading or trailing zeros.
                    pairs.push(KeyValuePair::new(
                        key.trim_matches(char::from(0)).to_string(),
                        value.trim_matches(char::from(0)).to_string(),
                    ))
                }
                Ok(Node::new(NodeType::Leaf(pairs), is_root, parent_offset))
            }

            NodeType::Unexpected => Err(Error::UnexpectedError),
        }
    }
}

////////////////////
///              ///
///  Unit Tests. ///
///              ///
////////////////////

#[cfg(test)]
mod tests {
    use crate::error::Error;
    use crate::node::{
        Node, Page, INTERNAL_NODE_HEADER_SIZE, KEY_SIZE, LEAF_NODE_HEADER_SIZE, PTR_SIZE,
        VALUE_SIZE,
    };
    use crate::node_type::{Key, NodeType};
    use crate::page_layout::PAGE_SIZE;
    use std::convert::TryFrom;

    #[test]
    fn page_to_node_works_for_leaf_node() -> Result<(), Error> {
        const DATA_LEN: usize = LEAF_NODE_HEADER_SIZE + KEY_SIZE + VALUE_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x02, // Leaf Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, // Number of Key-Value pairs.
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];
        let mut page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) {
            *to = *from
        }

        let node = Node::try_from(Page::new(page))?;

        assert_eq!(node.is_root, true);
        Ok(())
    }

    #[test]
    fn page_to_node_works_for_internal_node() -> Result<(), Error> {
        use crate::node_type::Key;
        const DATA_LEN: usize = INTERNAL_NODE_HEADER_SIZE + 3 * PTR_SIZE + 2 * KEY_SIZE;
        let page_data: [u8; DATA_LEN] = [
            0x01, // Is-Root byte.
            0x01, // Internal Node type byte.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // Parent offset.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, // Number of children.
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, // 4096  (2nd Page)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x20, 0x00, // 8192  (3rd Page)
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x00, // 12288 (4th Page)
            0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x00, 0x00, 0x00, 0x00, 0x00, // "hello"
            0x77, 0x6f, 0x72, 0x6c, 0x64, 0x00, 0x00, 0x00, 0x00, 0x00, // "world"
        ];
        let junk: [u8; PAGE_SIZE - DATA_LEN] = [0x00; PAGE_SIZE - DATA_LEN];

        // Concatenate the two arrays; page_data and junk.
        let mut page = [0x00; PAGE_SIZE];
        for (to, from) in page.iter_mut().zip(page_data.iter().chain(junk.iter())) {
            *to = *from
        }

        let node = Node::try_from(Page::new(page))?;

        if let NodeType::Internal(_, keys) = node.node_type {
            assert_eq!(keys.len(), 2);

            let Key(first_key) = match keys.get(0) {
                Some(key) => key,
                None => return Err(Error::UnexpectedError),
            };
            assert_eq!(first_key, "hello");

            let Key(second_key) = match keys.get(1) {
                Some(key) => key,
                None => return Err(Error::UnexpectedError),
            };
            assert_eq!(second_key, "world");
            return Ok(());
        }

        Err(Error::UnexpectedError)
    }

    #[test]
    fn split_leaf_works() -> Result<(), Error> {
        use crate::node::Node;
        use crate::node_type::KeyValuePair;
        let mut node = Node::new(
            NodeType::Leaf(vec![
                KeyValuePair::new("foo".to_string(), "bar".to_string()),
                KeyValuePair::new("lebron".to_string(), "james".to_string()),
                KeyValuePair::new("ariana".to_string(), "grande".to_string()),
            ]),
            true,
            None,
        );

        let (median, sibling) = node.split(2)?;
        assert_eq!(median, Key("lebron".to_string()));
        assert_eq!(
            node.node_type,
            NodeType::Leaf(vec![
                KeyValuePair {
                    key: "foo".to_string(),
                    value: "bar".to_string()
                },
                KeyValuePair {
                    key: "lebron".to_string(),
                    value: "james".to_string()
                }
            ])
        );
        assert_eq!(
            sibling.node_type,
            NodeType::Leaf(vec![KeyValuePair::new(
                "ariana".to_string(),
                "grande".to_string()
            )])
        );
        Ok(())
    }

    #[test]
    fn split_internal_works() -> Result<(), Error> {
        use crate::node::Node;
        use crate::node_type::NodeType;
        use crate::node_type::{Key, Offset};
        use crate::page_layout::PAGE_SIZE;
        let mut node = Node::new(
            NodeType::Internal(
                vec![
                    Offset(PAGE_SIZE),
                    Offset(PAGE_SIZE * 2),
                    Offset(PAGE_SIZE * 3),
                    Offset(PAGE_SIZE * 4),
                ],
                vec![
                    Key("foo bar".to_string()),
                    Key("lebron".to_string()),
                    Key("ariana".to_string()),
                ],
            ),
            true,
            None,
        );

        let (median, sibling) = node.split(2)?;
        assert_eq!(median, Key("lebron".to_string()));
        assert_eq!(
            node.node_type,
            NodeType::Internal(
                vec![Offset(PAGE_SIZE), Offset(PAGE_SIZE * 2)],
                vec![Key("foo bar".to_string())]
            )
        );
        assert_eq!(
            sibling.node_type,
            NodeType::Internal(
                vec![Offset(PAGE_SIZE * 3), Offset(PAGE_SIZE * 4)],
                vec![Key("ariana".to_string())]
            )
        );
        Ok(())
    }
}
