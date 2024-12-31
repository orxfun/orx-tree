use super::{IterKindCore, QueueElement};
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    TreeVariant,
};
use alloc::collections::VecDeque;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Breadth first search iterator.
/// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
pub struct Bfs<
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
    pub(super) col: &'a SelfRefCol<V, M, P>,
    pub(super) queue: S,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P, S> From<Bfs<'a, K, V, M, P, S>> for (&'a SelfRefCol<V, M, P>, S)
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<VecDeque<K::QueueElement>>,
{
    fn from(value: Bfs<'a, K, V, M, P, S>) -> Self {
        (value.col, value.queue)
    }
}

impl<'a, K, V, M, P> Bfs<'a, K, V, M, P, VecDeque<K::QueueElement>>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a SelfRefCol<V, M, P>, root_ptr: NodePtr<V>) -> Self {
        let mut queue = VecDeque::new();
        queue.push_back(<K::QueueElement as QueueElement<V>>::from_root_ptr(
            root_ptr,
        ));
        Self {
            col,
            queue,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V, M, P> Bfs<'a, K, V, M, P, &'a mut VecDeque<K::QueueElement>>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new_with_queue(
        col: &'a SelfRefCol<V, M, P>,
        root_ptr: NodePtr<V>,
        queue: &'a mut VecDeque<K::QueueElement>,
    ) -> Self {
        queue.clear();
        queue.push_back(<K::QueueElement as QueueElement<V>>::from_root_ptr(
            root_ptr,
        ));
        Self {
            col,
            queue,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V, M, P, S> Iterator for Bfs<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<VecDeque<K::QueueElement>>,
{
    type Item = K::YieldElement;

    fn next(&mut self) -> Option<Self::Item> {
        let queue: &mut VecDeque<K::QueueElement> = self.queue.get_mut();
        queue.pop_front().map(|parent| {
            let children = K::children(&parent);
            queue.extend(children);
            K::element(self.col, &parent)
        })
    }
}
