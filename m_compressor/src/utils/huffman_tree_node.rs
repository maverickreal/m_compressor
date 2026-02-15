
use std::sync::atomic::{AtomicUsize, Ordering};

static NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn next_node_id() -> usize {
    NODE_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

#[derive(Debug, Eq, PartialEq)]
pub struct HuffmanTreeNode {
    pub weight: usize,
    pub id: usize,
    pub left: Option<Box<HuffmanTreeNode>>,
    pub right: Option<Box<HuffmanTreeNode>>,
    pub symbol: Option<usize>,
}

impl HuffmanTreeNode {
    pub fn new_leaf(weight: usize, symbol: usize) -> Self {
        Self {
            weight,
            id: next_node_id(),
            left: None,
            right: None,
            symbol: Some(symbol),
        }
    }

    pub fn new_internal(left: HuffmanTreeNode, right: HuffmanTreeNode) -> Self {
        Self {
            weight: left.weight + right.weight,
            id: next_node_id(),
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            symbol: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}

impl Ord for HuffmanTreeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.weight
            .cmp(&other.weight)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for HuffmanTreeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
