use super::{Dfs, IterKindCore};
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    TreeVariant,
};
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::MemoryPolicy;

/// A mutable depth first search iterator.
/// This traversal also known as "pre-order" ([wiki](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order_implementation)).
pub struct DfsMut<
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
    dfs: Dfs<'a, K, V, M, P, S>,
}

impl<'a, K, V, M, P, S> From<Dfs<'a, K, V, M, P, S>> for DfsMut<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::QueueElement>>,
{
    fn from(dfs: Dfs<'a, K, V, M, P, S>) -> Self {
        Self { dfs }
    }
}

impl<'a, K, V, M, P, S> Iterator for DfsMut<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<Vec<K::QueueElement>>,
{
    type Item = K::YieldElementMut;

    fn next(&mut self) -> Option<Self::Item> {
        self.dfs.stack.get_mut().pop().map(|parent| {
            let children = K::children(&parent);
            self.dfs.stack.get_mut().extend(children);
            K::element_mut(self.dfs.col, &parent)
        })
    }
}
