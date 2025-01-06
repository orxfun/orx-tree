use crate::iter::{DfsBfsIterKind, QueueElement};
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    TreeVariant,
};
use alloc::vec::Vec;
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
    S = Vec<<K as DfsBfsIterKind<'a, V, M, P>>::QueueElement>,
> where
    K: DfsBfsIterKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::QueueElement>>,
{
    pub(super) col: &'a SelfRefCol<V, M, P>,
    pub(super) stack: S,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P> DfsIter<'a, K, V, M, P, &'a mut Vec<K::QueueElement>>
where
    K: DfsBfsIterKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new_using(
        col: &'a SelfRefCol<V, M, P>,
        root_ptr: NodePtr<V>,
        stack: &'a mut Vec<K::QueueElement>,
    ) -> Self {
        stack.clear();
        stack.push(<K::QueueElement as QueueElement<V>>::from_root_ptr(
            root_ptr,
        ));
        Self {
            col,
            stack,
            phantom: PhantomData,
        }
    }
}

// iterator

impl<'a, K, V, M, P, S> Iterator for DfsIter<'a, K, V, M, P, S>
where
    K: DfsBfsIterKind<'a, V, M, P>,
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<Vec<K::QueueElement>>,
{
    type Item = K::YieldElement;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.get_mut().pop().map(|parent| {
            let children = K::children_rev(&parent);
            self.stack.get_mut().extend(children);
            K::element(self.col, &parent)
        })
    }
}
