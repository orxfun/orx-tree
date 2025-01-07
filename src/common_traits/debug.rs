use crate::{helpers::N, memory::TreeMemoryPolicy, Node, NodeMut, TreeVariant};
use core::fmt::Debug;
use orx_pinned_vec::PinnedVec;

impl<V, M, P> Debug for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("Node").finish()
    }
}

impl<V, M, P> Debug for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("NodeMut").finish()
    }
}
