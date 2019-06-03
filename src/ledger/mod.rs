pub mod parse;
pub mod stats;

#[derive(Debug, PartialEq)]
enum Node {
    Valid(ValidNode),
    Invalid,
}

#[derive(Debug, PartialEq)]
struct ValidNode {
    left: Option<usize>,  // 0-indexed node ref
    right: Option<usize>, // 0-indexed node ref
    timestamp: u64,
}

impl Node {
    #[allow(dead_code)]
    fn new(left: usize, right: usize, timestamp: u64) -> Self {
        Node::Valid(ValidNode {
            left: Some(left),
            right: Some(right),
            timestamp,
        })
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct Ledger(Vec<Node>);
