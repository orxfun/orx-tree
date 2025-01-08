use super::bfs_enumeration::BreadthFirstEnumeration;
use super::iter_ptr::BfsIterPtr;
use super::queue::{Item, Queue};
use crate::helpers::Col;
use crate::memory::MemoryPolicy;
use crate::pinned_storage::PinnedStorage;
use crate::traversal::node_item::NodeItem;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_self_or::SoM;

pub struct BfsIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: BreadthFirstEnumeration,
    S: SoM<Queue<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    col: &'a Col<V, M, P>,
    iter: BfsIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> From<(&'a Col<V, M, P>, BfsIterPtr<V, E, S>)>
    for BfsIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: BreadthFirstEnumeration,
    S: SoM<Queue<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    fn from((col, iter): (&'a Col<V, M, P>, BfsIterPtr<V, E, S>)) -> Self {
        Self {
            col,
            iter,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, E, D> Clone for BfsIterRef<'a, V, M, P, E, Queue<V, E>, D>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: BreadthFirstEnumeration,
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

impl<'a, V, M, P, E, S, D> Iterator for BfsIterRef<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: BreadthFirstEnumeration,
    S: SoM<Queue<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|element: Item<V, E>| E::from_element_ptr(self.col, element))
    }
}
