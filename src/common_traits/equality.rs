use crate::{helpers::N, node_ref::NodeRefCore, Node, NodeMut, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

impl<V, M, P> PartialEq for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<V, M, P> PartialEq for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}
