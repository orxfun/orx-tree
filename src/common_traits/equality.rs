use crate::{helpers::N, node_ref::NodeRefCore, Node, NodeMut, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

impl<'a, V, M, P> PartialEq for Node<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<'a, V, M, P> PartialEq for NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}
