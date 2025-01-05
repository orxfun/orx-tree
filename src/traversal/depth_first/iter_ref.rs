use super::dfs_enumeration::DepthFirstEnumeration;
use super::iter_ptr::DfsIterPtr;
use super::{Item, Stack};
use crate::helpers::N;
use crate::traversal::node_item::NodeItem;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, SelfRefCol};

pub struct DfsIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    S: SoM<Stack<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    col: &'a SelfRefCol<V, M, P>,
    iter: DfsIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> From<(&'a SelfRefCol<V, M, P>, DfsIterPtr<V, E, S>)>
    for DfsIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    S: SoM<Stack<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    fn from((col, iter): (&'a SelfRefCol<V, M, P>, DfsIterPtr<V, E, S>)) -> Self {
        Self {
            col,
            iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, E, D> Clone for DfsIterRef<'a, V, M, P, E, Stack<V, E>, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    D: NodeItem<'a, V, M, P>,
    Item<V, E>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            col: self.col,
            iter: self.iter.clone(),
            phantom: self.phantom,
        }
    }
}

impl<'a, V, M, P, E, S, D> Iterator for DfsIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    S: SoM<Stack<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|element: Item<V, E>| E::from_element_ptr(self.col, element))
    }
}
