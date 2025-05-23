use crate::TreeVariant;
use crate::aliases::Col;
use crate::memory::{Auto, MemoryPolicy};
use crate::pinned_storage::{PinnedStorage, SplitRecursive};
use orx_selfref_col::NodePtr;

pub trait NodeItemMut<'a, V, M = Auto, P = SplitRecursive>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn from_ptr(col: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self;
}

impl<'a, V, M, P> NodeItemMut<'a, V, M, P> for &'a mut V::Item
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    #[inline(always)]
    fn from_ptr(_: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        let node = unsafe { &mut *node_ptr.ptr_mut() };
        node.data_mut().expect("active tree node has data")
    }
}
