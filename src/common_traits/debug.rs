use crate::{helpers::N, Node, NodeMut, TreeVariant};
use core::fmt::Debug;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

impl<'a, V, M, P> Debug for Node<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("Node")
            .field("node_ptr", &self.node_ptr)
            .finish()
    }
}

impl<'a, V, M, P> Debug for NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("NodeMut")
            .field("node_ptr", &self.node_ptr)
            .finish()
    }
}
