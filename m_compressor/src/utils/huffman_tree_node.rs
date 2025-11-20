

#[derive(Debug, Ord, PartialEq, PartialOrd, Eq)]
pub struct HuffmanTreeNode {
    pub weight: usize,
    pub left: Option<Box<HuffmanTreeNode>>,
    pub right: Option<Box<HuffmanTreeNode>>,
    pub symbol: Option<usize>,
}

impl HuffmanTreeNode {
    pub fn new_leaf(weight: usize, symbol: usize) -> Self {
        Self {
            weight,
            left: None,
            right: None,
            symbol: Some(symbol),
        }
    }

    pub fn new_internal(left: HuffmanTreeNode, right: HuffmanTreeNode) -> Self {
        Self {
            weight: left.weight + right.weight,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            symbol: None,
        }
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }
}
