use super::states::States;
use super::PostOrderIterPtr;
use super::{iter_ptr::Item, post_enumeration::PostOrderEnumeration};
use crate::helpers::N;
use crate::traversal::node_item_mut::NodeItemMut;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, SelfRefCol};

pub struct PostOrderIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItemMut<'a, V, M, P>,
{
    col: &'a SelfRefCol<V, M, P>,
    iter: PostOrderIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> From<(&'a mut SelfRefCol<V, M, P>, PostOrderIterPtr<V, E, S>)>
    for PostOrderIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItemMut<'a, V, M, P>,
{
    fn from((col, iter): (&'a mut SelfRefCol<V, M, P>, PostOrderIterPtr<V, E, S>)) -> Self {
        Self {
            col,
            iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, E, S, D> Iterator for PostOrderIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItemMut<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(move |element: Item<V, E>| E::from_element_ptr_mut(self.col, element))
    }
}
