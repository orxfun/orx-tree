use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage, Node, NodeMut, TreeVariant};
use core::fmt::Debug;

impl<V, M, P> Debug for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("Node").finish()
    }
}

impl<V, M, P> Debug for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // TODO
        f.debug_struct("NodeMut").finish()
    }
}
