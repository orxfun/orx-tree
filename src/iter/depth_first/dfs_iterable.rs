use super::DfsIterMut;
use crate::iter::{IterMutOver, IterOver};
use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::{helpers::N, iter::DfsIter, node_ref::NodeRefCore, NodeMut, NodeRef, TreeVariant};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_selfref_col::MemoryPolicy;
use orx_split_vec::PinnedVec;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
pub struct DfsIterable<
    K: IterOver,
    V: TreeVariant,
    M: MemoryPolicy<V> = DefaultMemory<V>,
    P: PinnedVec<N<V>> = DefaultPinVec<V>,
> {
    stack: Vec<K::DfsBfsQueueElement<V>>,
    phantom: PhantomData<(M, P)>,
}

impl<K, V, M, P> Default for DfsIterable<K, V, M, P>
where
    V: TreeVariant,
    K: IterOver,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<K, V, M, P> DfsIterable<K, V, M, P>
where
    V: TreeVariant,
    K: IterOver,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Creates a depth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter<'a>(&'a mut self, root: &'a impl NodeRef<'a, V, M, P>) -> DfsIterOf<'a, V, K, M, P>
    where
        V: 'a,
    {
        DfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.stack)
    }

    /// Creates a mutable depth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_mut<'a>(
        &'a mut self,
        root: &'a mut NodeMut<'a, V, M, P>,
    ) -> DfsIterMutOf<'a, V, K, M, P>
    where
        V: 'a,
        K: IterMutOver,
    {
        DfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.stack).into()
    }
}

// type simplification of iterators

type DfsIterOf<'a, V, K, M, P> = DfsIter<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut Vec<<K as IterOver>::DfsBfsQueueElement<V>>,
>;

type DfsIterMutOf<'a, V, K, M, P> = DfsIterMut<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut Vec<<K as IterOver>::DfsBfsQueueElement<V>>,
>;
