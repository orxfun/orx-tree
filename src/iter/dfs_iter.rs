use super::{IterKindCore, QueueElement};
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    TreeVariant,
};
use alloc::{vec, vec::Vec};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Depth first search iterator.
/// This traversal also known as "pre-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
pub struct DfsIter<
    'a,
    K,
    V,
    M = DefaultMemory<V>,
    P = DefaultPinVec<V>,
    S = Vec<<K as IterKindCore<'a, V, M, P>>::QueueElement>,
> where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::QueueElement>>,
{
    pub(super) col: &'a SelfRefCol<V, M, P>,
    pub(super) queue: S,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P, S> From<DfsIter<'a, K, V, M, P, S>> for (&'a SelfRefCol<V, M, P>, S)
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::QueueElement>>,
{
    fn from(value: DfsIter<'a, K, V, M, P, S>) -> Self {
        (value.col, value.queue)
    }
}

impl<'a, K, V, M, P> DfsIter<'a, K, V, M, P, Vec<K::QueueElement>>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a SelfRefCol<V, M, P>, root_ptr: NodePtr<V>) -> Self {
        let root = <K::QueueElement as QueueElement<V>>::from_root_ptr(root_ptr);
        let queue = vec![root];
        Self {
            col,
            queue,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V, M, P> DfsIter<'a, K, V, M, P, &'a mut Vec<K::QueueElement>>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new_with_queue(
        col: &'a SelfRefCol<V, M, P>,
        root_ptr: NodePtr<V>,
        queue: &'a mut Vec<K::QueueElement>,
    ) -> Self {
        queue.clear();
        queue.push(<K::QueueElement as QueueElement<V>>::from_root_ptr(
            root_ptr,
        ));
        Self {
            col,
            queue,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V, M, P, S> Iterator for DfsIter<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<Vec<K::QueueElement>>,
{
    type Item = K::YieldElement;

    fn next(&mut self) -> Option<Self::Item> {
        self.queue.get_mut().pop().map(|parent| {
            let children = K::children_rev(&parent);
            self.queue.get_mut().extend(children);
            K::element(self.col, &parent)
        })
    }
}
