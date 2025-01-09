use crate::helpers::Col;
use crate::memory::{Auto, MemoryPolicy};
use crate::pinned_storage::{PinnedStorage, SplitRecursive};
use crate::Node;
use crate::TreeVariant;
use orx_selfref_col::NodePtr;

pub trait NodeItem<'a, V, M = Auto, P = SplitRecursive>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn from_ptr(col: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self;

    #[cfg(test)]
    fn node_data(&self) -> &V::Item;
}

impl<'a, V, M, P> NodeItem<'a, V, M, P> for Node<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    #[inline(always)]
    fn from_ptr(col: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Node::new(col, node_ptr)
    }

    #[cfg(test)]
    #[inline(always)]
    fn node_data(&self) -> &V::Item {
        use crate::NodeRef;
        self.data()
    }
}

impl<'a, V, M, P> NodeItem<'a, V, M, P> for &'a V::Item
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    #[inline(always)]
    fn from_ptr(_: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        let node = unsafe { &*node_ptr.ptr() };
        node.data().expect("active tree node has data")
    }

    #[cfg(test)]
    #[inline(always)]
    fn node_data(&self) -> &V::Item {
        self
    }
}

impl<'a, V, M, P> NodeItem<'a, V, M, P> for NodePtr<V>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    #[inline(always)]
    fn from_ptr(_: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        node_ptr
    }

    #[cfg(test)]
    #[inline(always)]
    fn node_data(&self) -> &V::Item {
        unsafe { &*self.ptr() }
            .data()
            .expect("active tree node has data")
    }
}
