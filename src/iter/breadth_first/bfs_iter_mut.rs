use super::BfsIter;
use crate::iter::IterKindCore;
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    TreeVariant,
};
use alloc::collections::VecDeque;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::MemoryPolicy;

/// A mutable breadth first search iterator.
/// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
pub struct BfsIterMut<
    'a,
    K,
    V,
    M = DefaultMemory<V>,
    P = DefaultPinVec<V>,
    S = VecDeque<<K as IterKindCore<'a, V, M, P>>::QueueElement>,
> where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<VecDeque<K::QueueElement>>,
{
    bfs: BfsIter<'a, K, V, M, P, S>,
}

impl<'a, K, V, M, P, S> From<BfsIter<'a, K, V, M, P, S>> for BfsIterMut<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<VecDeque<K::QueueElement>>,
{
    fn from(bfs: BfsIter<'a, K, V, M, P, S>) -> Self {
        Self { bfs }
    }
}

impl<'a, K, V, M, P, S> Iterator for BfsIterMut<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<VecDeque<K::QueueElement>>,
{
    type Item = K::YieldElementMut;

    fn next(&mut self) -> Option<Self::Item> {
        let queue: &mut VecDeque<K::QueueElement> = self.bfs.queue.get_mut();
        queue.pop_front().map(|parent| {
            let children = K::children(&parent);
            queue.extend(children);
            K::element_mut(self.bfs.col, &parent)
        })
    }
}
