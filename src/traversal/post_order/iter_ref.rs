use super::iter_ptr::Item;
use super::iter_ptr::PostOrderIterPtr;
use super::post_enumeration::PostOrderEnumeration;
use super::states::States;
use crate::helpers::N;
use crate::traversal::node_item::NodeItem;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, SelfRefCol};

pub struct PostOrderIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItem<'a, V, M, P>,
{
    col: &'a SelfRefCol<V, M, P>,
    iter: PostOrderIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> From<(&'a SelfRefCol<V, M, P>, PostOrderIterPtr<V, E, S>)>
    for PostOrderIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItem<'a, V, M, P>,
{
    fn from((col, iter): (&'a SelfRefCol<V, M, P>, PostOrderIterPtr<V, E, S>)) -> Self {
        Self {
            col,
            iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, E, D> Clone for PostOrderIterRef<'a, V, M, P, E, States<V>, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
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

impl<'a, V, M, P, E, S, D> Iterator for PostOrderIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItem<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|element: Item<V, E>| E::from_element_ptr(self.col, element))
    }
}
