use crate::{
    helpers::{Col, N},
    memory::{Auto, TreeMemoryPolicy},
    node_ref::NodeRefCore,
    tree::DefaultPinVec,
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::NodePtr;

/// A node of the tree.
pub struct Node<'a, V, M = Auto, P = DefaultPinVec<V>>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    col: &'a Col<V, M, P>,
    node_ptr: NodePtr<V>,
}

impl<V, M, P> Clone for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn clone(&self) -> Self {
        Self {
            col: self.col,
            node_ptr: self.node_ptr.clone(),
        }
    }
}

impl<'a, V, M, P> Node<'a, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    // helpers

    pub(crate) fn new(col: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Self { col, node_ptr }
    }
}

impl<'a, V, M, P> NodeRefCore<'a, V, M, P> for Node<'a, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn col(&self) -> &Col<V, M, P> {
        self.col
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.node_ptr
    }
}
