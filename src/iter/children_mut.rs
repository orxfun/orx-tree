use crate::{helpers::N, node_mut::NodeMutDown, tree_variant::RefsChildren, NodeMut, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Mutable children iterator.
pub struct ChildrenMutIter<'a, 'b, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    'a: 'b,
{
    // node_ptr: *const N<V>,
    col: &'a mut SelfRefCol<V, M, P>,
    children_ptr: <V::Children as RefsChildren<V>>::ChildrenPtrIter<'b>,
}

impl<'a, 'b, V, M, P> ChildrenMutIter<'a, 'b, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    'a: 'b,
{
    pub(crate) fn new(col: &'a mut SelfRefCol<V, M, P>, node_ptr: *const N<V>) -> Self {
        let node = unsafe { &*node_ptr };
        let children_ptr = node.next().children_ptr();

        Self { children_ptr, col }
    }

    fn next_child(&mut self, child_ptr: NodePtr<V>) -> NodeMut<'b, V, M, P, NodeMutDown> {
        let col_mut = unsafe { &mut *(self.col as *mut SelfRefCol<V, M, P>) };
        NodeMut::new(col_mut, child_ptr)
    }
}

impl<'a, 'b, V, M, P> Iterator for ChildrenMutIter<'a, 'b, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    'a: 'b,
{
    type Item = NodeMut<'b, V, M, P, NodeMutDown>;

    fn next(&mut self) -> Option<Self::Item> {
        self.children_ptr.next().map(|p| self.next_child(p.clone()))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.children_ptr.size_hint()
    }
}

impl<'a, 'b, V, M, P> ExactSizeIterator for ChildrenMutIter<'a, 'b, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    'a: 'b,
{
    fn len(&self) -> usize {
        self.children_ptr.len()
    }
}

impl<'a, 'b, V, M, P> DoubleEndedIterator for ChildrenMutIter<'a, 'b, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    'a: 'b,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.children_ptr
            .next_back()
            .map(|p| self.next_child(p.clone()))
    }
}
