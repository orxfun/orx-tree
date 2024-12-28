use crate::{helpers::N, TreeNodeRef, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// Mutable reference to a tree node.
pub trait TreeNodeMut<V, M, P>: TreeNodeRef<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Returns a reference to the data of the node.
    fn data_mut<'a>(&'a mut self) -> &'a mut V::Item
    where
        V: 'a,
    {
        unsafe { &mut *self.node_ptr().ptr_mut() }
            .data_mut()
            .expect("node holding a tree reference cannot be closed")
    }
}
