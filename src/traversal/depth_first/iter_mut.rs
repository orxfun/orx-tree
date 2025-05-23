use super::dfs_enumeration::DepthFirstEnumeration;
use super::iter_ptr::DfsIterPtr;
use super::stack::Item;
use crate::TreeVariant;
use crate::aliases::Col;
use crate::memory::MemoryPolicy;
use crate::pinned_storage::PinnedStorage;
use crate::traversal::node_item_mut::NodeItemMut;
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_self_or::SoM;

pub struct DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
    D: NodeItemMut<'a, V, M, P>,
{
    col: &'a Col<V, M, P>,
    iter: DfsIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

impl<'a, V, M, P, E, S, D> DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
    D: NodeItemMut<'a, V, M, P>,
{
    /// # Safety
    ///
    /// We are creating a mutable iterator over nodes of the collection `col`.
    /// This is safe only when the second argument `iter` makes sure that there exists only one mutable
    /// reference to the collection.
    ///
    /// This is the case how this method is used, as follows:
    /// * Mutable iterators are created through the `Dfs` traverser's `TraverserMut::iter_mut` method.
    /// * This method requires a mutable reference to a mutable node `NodeMut` which is guaranteed to
    ///   be the only reference to the collection.
    /// * Finally, this iterator's lifetime is equal to the borrow duration of the mutable node.
    pub(crate) unsafe fn from((col, iter): (&'a Col<V, M, P>, DfsIterPtr<V, E, S>)) -> Self {
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
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
    D: NodeItemMut<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(move |element: Item<V, E>| E::from_element_ptr_mut(self.col, element))
    }
}
