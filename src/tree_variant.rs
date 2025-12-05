#[cfg(feature = "parallel")]
use orx_parallel::*;
use orx_selfref_col::{
    MemoryReclaimer, NodePtr, Refs, RefsArrayLeftMost, RefsSingle, RefsVec, Variant,
    references::iter::ArrayLeftMostPtrIter,
};

/// Variant of a tree.
pub trait TreeVariant:
    Variant<Ends = RefsSingle<Self>, Prev = RefsSingle<Self>, Next = Self::Children> + Sync
{
    /// Memory reclaimer of the tree.
    type Reclaimer: MemoryReclaimer<Self> + Sync;

    /// Children references of the tree nodes.
    type Children: RefsChildren<Self> + Refs;
}

// children

pub trait RefsChildren<V: Variant> {
    type ChildrenPtrIter<'a>: ExactSizeIterator<Item = &'a NodePtr<V>>
        + DoubleEndedIterator
        + Default
    where
        V: 'a,
        Self: 'a;

    fn num_children(&self) -> usize;

    fn children_ptr(&self) -> Self::ChildrenPtrIter<'_>;

    #[cfg(feature = "parallel")]
    fn children_ptr_par<'a>(&'a self) -> impl ParIter<Item = &'a NodePtr<V>>
    where
        V: 'a,
        V::Item: Send + Sync;

    fn get_ptr(&self, i: usize) -> Option<NodePtr<V>>;

    // mut

    fn push(&mut self, node_ptr: NodePtr<V>);

    fn insert(&mut self, position: usize, node_ptr: NodePtr<V>);

    fn replace_with(&mut self, old_node_ptr: NodePtr<V>, new_node_ptr: NodePtr<V>)
    -> Option<usize>;
}

impl<V: Variant> RefsChildren<V> for RefsVec<V> {
    type ChildrenPtrIter<'a>
        = core::slice::Iter<'a, NodePtr<V>>
    where
        V: 'a,
        Self: 'a;

    #[inline(always)]
    fn num_children(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn children_ptr(&self) -> Self::ChildrenPtrIter<'_> {
        self.iter()
    }

    #[cfg(feature = "parallel")]
    fn children_ptr_par<'a>(&'a self) -> impl ParIter<Item = &'a NodePtr<V>>
    where
        V: 'a,
        V::Item: Send + Sync,
    {
        self.as_slice().par()
    }

    #[inline(always)]
    fn get_ptr(&self, i: usize) -> Option<NodePtr<V>> {
        self.get(i)
    }

    #[inline(always)]
    fn push(&mut self, node_ptr: NodePtr<V>) {
        self.push(node_ptr);
    }

    #[inline(always)]
    fn insert(&mut self, position: usize, node_ptr: NodePtr<V>) {
        RefsVec::insert(self, position, node_ptr);
    }

    #[inline(always)]
    fn replace_with(
        &mut self,
        old_node_ptr: NodePtr<V>,
        new_node_ptr: NodePtr<V>,
    ) -> Option<usize> {
        RefsVec::replace_with(self, old_node_ptr, new_node_ptr)
    }
}

impl<const D: usize, V: Variant> RefsChildren<V> for RefsArrayLeftMost<D, V> {
    type ChildrenPtrIter<'a>
        = ArrayLeftMostPtrIter<'a, V>
    where
        V: 'a,
        Self: 'a;

    #[inline(always)]
    fn num_children(&self) -> usize {
        self.len()
    }

    #[inline(always)]
    fn children_ptr(&self) -> Self::ChildrenPtrIter<'_> {
        self.iter()
    }

    #[cfg(feature = "parallel")]
    fn children_ptr_par<'a>(&'a self) -> impl ParIter<Item = &'a NodePtr<V>>
    where
        V: 'a,
        V::Item: Send + Sync,
    {
        self.as_slice().par().map(|x| {
            x.as_ref()
                .expect("all elements of RefsArrayLeftMost::as_slice are of Some variant")
        })
    }

    #[inline(always)]
    fn get_ptr(&self, i: usize) -> Option<NodePtr<V>> {
        self.get(i)
    }

    #[inline(always)]
    fn push(&mut self, node_ptr: NodePtr<V>) {
        self.push(node_ptr);
    }

    #[inline(always)]
    fn insert(&mut self, position: usize, node_ptr: NodePtr<V>) {
        RefsArrayLeftMost::insert(self, position, node_ptr);
    }

    #[inline(always)]
    fn replace_with(
        &mut self,
        old_node_ptr: NodePtr<V>,
        new_node_ptr: NodePtr<V>,
    ) -> Option<usize> {
        RefsArrayLeftMost::replace_with(self, old_node_ptr, new_node_ptr)
    }
}
