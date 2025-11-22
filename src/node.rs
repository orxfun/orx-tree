use crate::{
    TreeVariant,
    aliases::Col,
    memory::{Auto, MemoryPolicy},
    node_ref::NodeRefCore,
    pinned_storage::{PinnedStorage, SplitRecursive},
};
use orx_selfref_col::NodePtr;

/// A node of the tree.
pub struct Node<'a, V, M = Auto, P = SplitRecursive>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    col: &'a Col<V, M, P>,
    node_ptr: NodePtr<V>,
}

// SAFETY: Required for enabling `NodeRef::walk_with_par`.
// Notice that `Node` does not expose any methods other than implementing `NodeRef`,
// and all node ref methods are thread safe without data race risks.
unsafe impl<V, M, P> Send for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Send,
{
}
// SAFETY: Required for enabling `NodeRef::walk_with_par`.
// Notice that `Node` does not expose any methods other than implementing `NodeRef`,
// and all node ref methods are thread safe without data race risks.
unsafe impl<V, M, P> Sync for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Sync,
{
}

impl<V, M, P> Clone for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn clone(&self) -> Self {
        Self {
            col: self.col,
            node_ptr: self.node_ptr,
        }
    }
}

impl<'a, V, M, P> Node<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    // helpers

    pub(crate) fn new(col: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Self { col, node_ptr }
    }
}

impl<'a, V, M, P> NodeRefCore<'a, V, M, P> for Node<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    #[inline(always)]
    fn col(&self) -> &'a Col<V, M, P> {
        self.col
    }

    #[inline(always)]
    fn node_ptr(&self) -> NodePtr<V> {
        self.node_ptr
    }
}
