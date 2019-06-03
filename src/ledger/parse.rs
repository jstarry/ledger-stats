use super::{Ledger, Node, ValidNode};
use failure::Fail;
use std::io::BufRead;
use std::num::ParseIntError;

#[derive(Debug, PartialEq, Fail)]
enum NodeParseError {
    #[fail(display = "node 0 is not a valid node")]
    ZeroNode,
    #[fail(display = "nodes cannot reference future nodes")]
    FutureRef,
    #[fail(display = "nodes cannot only reference themselves")]
    SelfRef,
    #[fail(display = "node needs one valid ref")]
    NoValidRef,
    #[fail(display = "node has invalid timestamp")]
    InvalidTimestamp,
    #[fail(display = "failed to parse node")]
    ParseError,
}

impl From<ParseIntError> for NodeParseError {
    fn from(_: ParseIntError) -> Self {
        NodeParseError::ParseError
    }
}

impl Ledger {
    pub fn parse<R>(mut reader: R) -> Self
    where
        R: BufRead,
    {
        let mut num_nodes_string = String::new();
        reader.read_line(&mut num_nodes_string).expect("read error");
        let num_nodes_string = num_nodes_string.trim();
        assert!(!num_nodes_string.is_empty(), "input error");
        let num_nodes = num_nodes_string.parse::<usize>().expect("parse error");

        let mut nodes: Vec<Node> = vec![Node::new(0, 0, 0)];
        let mut node_string = String::new();
        for _ in 0..num_nodes {
            if reader.read_line(&mut node_string).expect("read error") == 0 {
                panic!("unexpected end of input");
            }
            if let Ok(node) = Ledger::parse_node(&node_string, &nodes) {
                nodes.push(Node::Valid(node));
            } else {
                nodes.push(Node::Invalid);
            }
            node_string.clear();
        }

        // Check to make sure input is finished
        while reader.read_line(&mut node_string).expect("read error") != 0 {
            if node_string.trim().len() > 0 {
                panic!("expected end of input");
            }
        }

        // Exclude origin node
        Ledger(nodes.split_off(1))
    }

    fn parse_node(line: &str, nodes: &Vec<Node>) -> Result<ValidNode, NodeParseError> {
        let next_index = nodes.len() + 1;
        let mut token_iter = line.split_whitespace();
        let (mut left_index, mut right_index, timestamp) = (
            token_iter.next().expect("input error").parse::<usize>()?,
            token_iter.next().expect("input error").parse::<usize>()?,
            token_iter.next().expect("input error").parse::<u64>()?,
        );

        if left_index == 0 || right_index == 0 {
            return Err(NodeParseError::ZeroNode);
        }

        // change to zero index
        left_index -= 1;
        right_index -= 1;

        if left_index > next_index || right_index > next_index {
            return Err(NodeParseError::FutureRef);
        }

        if left_index == next_index && right_index == next_index {
            return Err(NodeParseError::SelfRef);
        }

        if Self::invalid_timestamp(left_index, timestamp, nodes)
            || Self::invalid_timestamp(right_index, timestamp, nodes)
        {
            return Err(NodeParseError::InvalidTimestamp);
        }

        let left = Self::ok_ref(left_index, nodes);
        let right = Self::ok_ref(right_index, nodes);
        if left.is_none() && right.is_none() {
            return Err(NodeParseError::NoValidRef);
        }

        Ok(ValidNode {
            left,
            right,
            timestamp,
        })
    }

    fn ok_ref(node_ref: usize, nodes: &Vec<Node>) -> Option<usize> {
        if node_ref < nodes.len() {
            if nodes[node_ref] == Node::Invalid {
                return None;
            }
        }

        Some(node_ref)
    }

    fn invalid_timestamp(node_ref: usize, timestamp: u64, nodes: &Vec<Node>) -> bool {
        if node_ref < nodes.len() {
            if let Node::Valid(node) = &nodes[node_ref] {
                if timestamp < node.timestamp {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    #[test]
    fn test_parse_node() {
        let node = Ledger::parse_node("1 1 0", &vec![]).expect("node should be valid");
        assert_eq!(Node::Valid(node), Node::new(0, 0, 0));
    }

    #[test]
    fn test_parse_node_zero_node() {
        let err = Ledger::parse_node("0 0 0", &vec![]).err().unwrap();
        assert_eq!(err, NodeParseError::ZeroNode);
    }

    #[test]
    fn test_parse_node_future_ref() {
        let err = Ledger::parse_node("1 3 0", &vec![]).err().unwrap();
        assert_eq!(err, NodeParseError::FutureRef);
    }

    #[test]
    fn test_parse_node_self_ref() {
        let err = Ledger::parse_node("2 2 0", &vec![]).err().unwrap();
        assert_eq!(err, NodeParseError::SelfRef);
    }

    #[test]
    fn test_parse_node_no_valid_ref() {
        let nodes = vec![Node::Invalid, Node::Invalid];
        let err = Ledger::parse_node("1 2 0", &nodes).err().unwrap();
        assert_eq!(err, NodeParseError::NoValidRef);
    }

    #[test]
    fn test_parse_node_early_timestamp() {
        let nodes = vec![Node::new(0, 0, 1)];
        let err = Ledger::parse_node("1 1 0", &nodes).err().unwrap();
        assert_eq!(err, NodeParseError::InvalidTimestamp);
    }

    #[test]
    fn test_parse_node_invalid() {
        let err = Ledger::parse_node("a 1 0", &vec![]).err().unwrap();
        assert_eq!(err, NodeParseError::ParseError);
    }

    #[test]
    fn test_parse_node_invalid_timestamp() {
        let err = Ledger::parse_node("1 1 1111111111111111111111111111111111111", &vec![])
            .err()
            .unwrap();
        assert_eq!(err, NodeParseError::ParseError);
    }

    #[test]
    fn test_parse() {
        let database_txt = "5
        1 1 0
        1 2 0
        2 2 1
        3 3 2
        3 4 3";

        let db = Ledger::parse(BufReader::new(database_txt.as_bytes()));
        assert_eq!(
            db,
            Ledger(vec![
                Node::new(0, 0, 0),
                Node::new(0, 1, 0),
                Node::new(1, 1, 1),
                Node::new(2, 2, 2),
                Node::new(2, 3, 3),
            ])
        );
    }

    #[test]
    fn test_parse_handle_invalids() {
        let database_txt = "5
        1 1 1
        1 2 0
        2 2 1
        3 3 2
        3 4 3";

        let db = Ledger::parse(BufReader::new(database_txt.as_bytes()));
        assert_eq!(
            db,
            Ledger(vec![
                Node::new(0, 0, 1),
                Node::Invalid,
                Node::new(1, 1, 1),
                Node::Invalid,
                Node::Valid(ValidNode {
                    left: None,
                    right: Some(3),
                    timestamp: 3,
                }),
            ])
        );
    }

    #[test]
    fn test_parse_cycle() {
        let database_txt = "3
        1 4 1
        1 2 2
        1 3 3";

        let db = Ledger::parse(BufReader::new(database_txt.as_bytes()));
        assert_eq!(
            db,
            Ledger(vec![
                Node::Invalid,
                Node::Valid(ValidNode {
                    left: Some(0),
                    right: None,
                    timestamp: 2,
                }),
                Node::new(0, 2, 3),
            ])
        );
    }

    #[test]
    fn test_parse_empty() {
        let database_txt = "0";
        let db = Ledger::parse(BufReader::new(database_txt.as_bytes()));
        assert_eq!(db, Ledger::default());
    }

    #[test]
    #[should_panic(expected = "expected end of input")]
    fn test_parse_extra_line() {
        let database_txt = "0
        1 1 0";
        Ledger::parse(BufReader::new(database_txt.as_bytes()));
    }

    #[test]
    #[should_panic(expected = "unexpected end of input")]
    fn test_parse_missing_line() {
        let database_txt = "1";
        Ledger::parse(BufReader::new(database_txt.as_bytes()));
    }

    #[test]
    #[should_panic(expected = "parse error")]
    fn test_parse_err() {
        let database_txt = "1 abc";
        Ledger::parse(BufReader::new(database_txt.as_bytes()));
    }

    #[test]
    #[should_panic(expected = "input error")]
    fn test_parse_input_err() {
        Ledger::parse(std::io::empty());
    }

    #[test]
    #[should_panic(expected = "read error")]
    fn test_parse_read_err() {
        let database_txt: &[u8] = b"\xc3\x28"; // invalid utf8
        Ledger::parse(BufReader::new(database_txt));
    }
}
