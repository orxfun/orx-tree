use super::kind::IterKind;
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

/// A depth first search iterator; also known as "pre-order traversal" ([wiki](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order_implementation)).
pub struct DfsNodes<'a, K, V, M = DefaultMemory<V>, P = DefaultPinVec<V>, S = Vec<NodePtr<V>>>
where
    K: IterKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::StackElement>>,
{
    col: &'a SelfRefCol<V, M, P>,
    stack: S,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P, S> Iterator for DfsNodes<'a, K, V, M, P, S>
where
    K: IterKind<'a, V, M, P>,
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<Vec<K::StackElement>>,
{
    type Item = K::YieldElement;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.get_mut().pop().map(|parent| {
            let children = K::children(&parent);
            self.stack.get_mut().extend(children);
            K::element(self.col, &parent)
        })
    }
}
