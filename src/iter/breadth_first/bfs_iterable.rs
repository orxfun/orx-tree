use super::BfsIterMut;
use crate::iter::{IterMutOver, IterOver};
use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::{helpers::N, iter::BfsIter, node_ref::NodeRefCore, NodeMut, NodeRef, TreeVariant};
use alloc::collections::VecDeque;
use core::marker::PhantomData;
use orx_selfref_col::MemoryPolicy;
use orx_split_vec::PinnedVec;

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
pub struct BfsIterable<
    K: IterOver,
    V: TreeVariant,
    M: MemoryPolicy<V> = DefaultMemory<V>,
    P: PinnedVec<N<V>> = DefaultPinVec<V>,
> {
    queue: VecDeque<K::DfsBfsQueueElement<V>>,
    phantom: PhantomData<(M, P)>,
}

impl<K, V, M, P> Default for BfsIterable<K, V, M, P>
where
    V: TreeVariant,
    K: IterOver,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
            phantom: PhantomData,
        }
    }
}

impl<K, V, M, P> BfsIterable<K, V, M, P>
where
    V: TreeVariant,
    K: IterOver,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Creates a breadth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the breadth-first-search will be rooted at this node:
    /// * root's breadth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter<'a>(&'a mut self, root: &'a impl NodeRef<'a, V, M, P>) -> BfsIterOf<'a, V, K, M, P>
    where
        V: 'a,
    {
        BfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.queue)
    }

    /// Creates a mutable breadth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the breadth-first-search will be rooted at this node:
    /// * root's breadth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_mut<'a>(
        &'a mut self,
        root: &'a mut NodeMut<'a, V, M, P>,
    ) -> BfsIterMutOf<'a, V, K, M, P>
    where
        V: 'a,
        K: IterMutOver,
    {
        BfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.queue).into()
    }
}

// type simplification of iterators

type BfsIterOf<'a, V, K, M, P> = BfsIter<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut VecDeque<<K as IterOver>::DfsBfsQueueElement<V>>,
>;

type BfsIterMutOf<'a, V, K, M, P> = BfsIterMut<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut VecDeque<<K as IterOver>::DfsBfsQueueElement<V>>,
>;
