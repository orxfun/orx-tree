use crate::helpers::N;
use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::TreeVariant;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait NodeItemMut<'a, V, M = DefaultMemory<V>, P = DefaultPinVec<V>>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn from_ptr(col: &'a SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self;
}

impl<'a, V, M, P> NodeItemMut<'a, V, M, P> for &'a mut V::Item
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn from_ptr(_: &'a SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        let node = unsafe { &mut *node_ptr.ptr_mut() };
        node.data_mut().expect("active tree node has data")
    }
}
