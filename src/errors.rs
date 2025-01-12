use orx_selfref_col::NodeIdxError;

/// Error variants that can be observed while swapping two subtrees rooted at two
/// nodes of the tree.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum NodeSwapError {
    /// At least one of the indices of the roots of the two subtrees is invalid.
    NodeIdxError(NodeIdxError),
    /// The first node is an ancestor of the second node; and hence, the subtrees
    /// intersect.
    /// The subtrees must be independent to have a valid swap operation.
    FirstNodeIsAncestorOfSecond,
    /// The second node is an ancestor of the first node; and hence, the subtrees
    /// intersect.
    /// The subtrees must be independent to have a valid swap operation.
    SecondNodeIsAncestorOfFirst,
}

impl From<NodeIdxError> for NodeSwapError {
    fn from(value: NodeIdxError) -> Self {
        Self::NodeIdxError(value)
    }
}
