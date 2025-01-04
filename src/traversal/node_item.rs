use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::{helpers::N, Node};
use crate::{NodeRef, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait NodeItem<'a, V, M = DefaultMemory<V>, P = DefaultPinVec<V>>: Clone
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn from_ptr(col: &'a SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self;

    fn node_data(&self) -> &V::Item;
}

impl<'a, V, M, P> NodeItem<'a, V, M, P> for Node<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn from_ptr(col: &'a SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Node::new(col, node_ptr)
    }

    #[inline(always)]
    fn node_data(&self) -> &<V>::Item {
        self.data()
    }
}

impl<'a, V, M, P> NodeItem<'a, V, M, P> for &'a V::Item
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn from_ptr(_: &'a SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        let node = unsafe { &*node_ptr.ptr() };
        node.data().expect("active tree node has data")
    }

    #[inline(always)]
    fn node_data(&self) -> &<V>::Item {
        self
    }
}

impl<'a, V, M, P> NodeItem<'a, V, M, P> for NodePtr<V>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn from_ptr(_: &'a SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        node_ptr
    }

    #[inline(always)]
    fn node_data(&self) -> &<V>::Item {
        unsafe { &*self.ptr() }
            .data()
            .expect("active tree node has data")
    }
}
