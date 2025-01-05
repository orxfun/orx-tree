use super::{dfs_enumeration::DepthFirstEnumeration, DfsIterPtr};
use super::{Item, Stack};
use crate::helpers::N;
use crate::traversal::node_item_mut::NodeItemMut;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, SelfRefCol};

pub struct DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    S: SoM<Stack<V, E>>,
    D: NodeItemMut<'a, V, M, P>,
{
    col: &'a SelfRefCol<V, M, P>,
    iter: DfsIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> From<(&'a SelfRefCol<V, M, P>, DfsIterPtr<V, E, S>)>
    for DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    S: SoM<Stack<V, E>>,
    D: NodeItemMut<'a, V, M, P>,
{
    fn from((col, iter): (&'a SelfRefCol<V, M, P>, DfsIterPtr<V, E, S>)) -> Self {
        Self {
            col,
            iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, E, S, D> Iterator for DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: DepthFirstEnumeration,
    S: SoM<Stack<V, E>>,
    D: NodeItemMut<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(move |element: Item<V, E>| E::from_element_ptr_mut(self.col, element))
    }
}
