use orx_selfref_col::{
    references::iter::ArrayLeftMostPtrIter, MemoryReclaimer, NodePtr, Refs, RefsArrayLeftMost,
    RefsSingle, RefsVec, Variant,
};

/// Variant of a tree.
pub trait TreeVariant:
    Variant<Ends = RefsSingle<Self>, Prev = RefsSingle<Self>, Next = Self::Children>
{
    /// Memory reclaimer of the tree.
    type Reclaimer: MemoryReclaimer<Self>;

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

    fn get_ptr(&self, i: usize) -> Option<&NodePtr<V>>;

    // mut

    fn push(&mut self, node_ptr: NodePtr<V>);
}

impl<V: Variant> RefsChildren<V> for RefsVec<V> {
    type ChildrenPtrIter<'a>
        = core::slice::Iter<'a, NodePtr<V>>
    where
        V: 'a,
        Self: 'a;

    fn num_children(&self) -> usize {
        self.len()
    }

    fn children_ptr(&self) -> Self::ChildrenPtrIter<'_> {
        self.iter()
    }

    fn get_ptr(&self, i: usize) -> Option<&NodePtr<V>> {
        self.get(i)
    }

    fn push(&mut self, node_ptr: NodePtr<V>) {
        self.push(node_ptr);
    }
}

impl<const D: usize, V: Variant> RefsChildren<V> for RefsArrayLeftMost<D, V> {
    type ChildrenPtrIter<'a>
        = ArrayLeftMostPtrIter<'a, V>
    where
        V: 'a,
        Self: 'a;

    fn num_children(&self) -> usize {
        self.len()
    }

    fn children_ptr(&self) -> Self::ChildrenPtrIter<'_> {
        self.iter()
    }

    fn get_ptr(&self, i: usize) -> Option<&NodePtr<V>> {
        self.get(i)
    }

    fn push(&mut self, node_ptr: NodePtr<V>) {
        self.push(node_ptr);
    }
}
