use crate::TreeVariant;
use orx_selfref_col::NodePtr;

/// Intermediate element of the tree iteration which are enqueued and dequeued during the iteration.
pub trait QueueElement<V: TreeVariant> {
    /// Creates the queue element for the root node.
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self;

    /// Gets the node pointer of the queue element.
    fn node_ptr(&self) -> &NodePtr<V>;
}

impl<V: TreeVariant> QueueElement<V> for NodePtr<V> {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        root_ptr
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        self
    }
}

impl<V: TreeVariant> QueueElement<V> for (usize, NodePtr<V>) {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        (0, root_ptr)
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.1
    }
}

impl<V: TreeVariant> QueueElement<V> for (usize, usize, NodePtr<V>) {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        (0, 0, root_ptr)
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.2
    }
}
