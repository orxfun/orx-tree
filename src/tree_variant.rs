use orx_selfref_col::{MemoryReclaimer, NodePtr, RefsArrayLeftMost, RefsSingle, RefsVec, Variant};

/// Variant of a tree.
pub trait TreeVariant:
    Variant<Ends = RefsSingle<Self>, Prev = RefsSingle<Self>, Next = Self::Children>
{
    /// Memory reclaimer of the tree.
    type Reclaimer: MemoryReclaimer<Self>;

    /// Children references of the tree nodes.
    type Children: RefsChildren<Self>;
}

// children

pub trait RefsChildren<V: Variant> {
    fn num_children(&self) -> usize;

    fn children_ptr<'a>(&'a self) -> impl ExactSizeIterator<Item = &'a NodePtr<V>>
    where
        V: 'a;

    fn get_ptr(&self, i: usize) -> Option<&NodePtr<V>>;

    // mut

    fn push(&mut self, node_ptr: NodePtr<V>);
}

impl<V: Variant> RefsChildren<V> for RefsVec<V> {
    fn num_children(&self) -> usize {
        self.len()
    }

    fn children_ptr<'a>(&'a self) -> impl ExactSizeIterator<Item = &'a NodePtr<V>>
    where
        V: 'a,
    {
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
    fn num_children(&self) -> usize {
        self.len()
    }

    fn children_ptr<'a>(&'a self) -> impl ExactSizeIterator<Item = &'a NodePtr<V>>
    where
        V: 'a,
    {
        self.iter()
    }

    fn get_ptr(&self, i: usize) -> Option<&NodePtr<V>> {
        self.get(i)
    }

    fn push(&mut self, node_ptr: NodePtr<V>) {
        self.push(node_ptr);
    }
}
