use super::{IterKindCore, StackElement};
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
pub struct Dfs<
    'a,
    K,
    V,
    M = DefaultMemory<V>,
    P = DefaultPinVec<V>,
    S = Vec<<K as IterKindCore<'a, V, M, P>>::StackElement>,
> where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::StackElement>>,
{
    col: &'a SelfRefCol<V, M, P>,
    stack: S,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P, S> From<Dfs<'a, K, V, M, P, S>> for (&'a SelfRefCol<V, M, P>, S)
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<K::StackElement>>,
{
    fn from(value: Dfs<'a, K, V, M, P, S>) -> Self {
        (value.col, value.stack)
    }
}

impl<'a, K, V, M, P> Dfs<'a, K, V, M, P, Vec<K::StackElement>>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a SelfRefCol<V, M, P>, root_ptr: NodePtr<V>) -> Self {
        let mut stack = Vec::new();
        stack.push(<K::StackElement as StackElement<V>>::from_root_ptr(
            root_ptr,
        ));
        Self {
            col,
            stack,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V, M, P> Dfs<'a, K, V, M, P, &'a mut Vec<K::StackElement>>
where
    K: IterKindCore<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new_with_stack(
        col: &'a SelfRefCol<V, M, P>,
        root_ptr: NodePtr<V>,
        stack: &'a mut Vec<K::StackElement>,
    ) -> Self {
        stack.get_mut().clear();
        stack.push(<K::StackElement as StackElement<V>>::from_root_ptr(
            root_ptr,
        ));
        Self {
            col,
            stack,
            phantom: PhantomData,
        }
    }
}

impl<'a, K, V, M, P, S> Iterator for Dfs<'a, K, V, M, P, S>
where
    K: IterKindCore<'a, V, M, P>,
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
