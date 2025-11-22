use crate::{
    MemoryPolicy, Node, TreeVariant, aliases::Col, node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
};
use orx_selfref_col::NodePtr;

/// An iterator which can traverse the tree arbitrarily in any direction where the walk direction
/// is determined by a custom `next_node` closure with signature `Fn(Node) -> Option(Node)`.
pub struct CustomWalkIterPtr<'a, V, M, P, F>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    F: Fn(Node<'a, V, M, P>) -> Option<Node<'a, V, M, P>>,
{
    col: &'a Col<V, M, P>,
    current: Option<NodePtr<V>>,
    next_node: F,
}

impl<'a, V, M, P, F> CustomWalkIterPtr<'a, V, M, P, F>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    F: Fn(Node<'a, V, M, P>) -> Option<Node<'a, V, M, P>>,
{
    pub(crate) fn new(col: &'a Col<V, M, P>, current: Option<NodePtr<V>>, next_node: F) -> Self {
        Self {
            col,
            current,
            next_node,
        }
    }
}

impl<'a, V, M, P, F> Iterator for CustomWalkIterPtr<'a, V, M, P, F>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    F: Fn(Node<'a, V, M, P>) -> Option<Node<'a, V, M, P>>,
{
    type Item = NodePtr<V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.clone().inspect(|current| {
            let node = Node::new(self.col, current.clone());
            self.current = (self.next_node)(node).map(|x| x.node_ptr());
        })
    }
}
