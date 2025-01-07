use super::iter_ptr::Item;
use super::iter_ptr::PostOrderIterPtr;
use super::post_enumeration::PostOrderEnumeration;
use super::states::States;
use crate::helpers::Col;
use crate::memory::TreeMemoryPolicy;
use crate::pinned_storage::PinnedStorage;
use crate::traversal::node_item::NodeItem;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_self_or::SoM;

pub struct PostOrderIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedStorage,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItem<'a, V, M, P>,
{
    col: &'a Col<V, M, P>,
    iter: PostOrderIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> From<(&'a Col<V, M, P>, PostOrderIterPtr<V, E, S>)>
    for PostOrderIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedStorage,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
    D: NodeItem<'a, V, M, P>,
{
    fn from((col, iter): (&'a Col<V, M, P>, PostOrderIterPtr<V, E, S>)) -> Self {
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
    M: TreeMemoryPolicy,
    P: PinnedStorage,
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
    M: TreeMemoryPolicy,
    P: PinnedStorage,
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
