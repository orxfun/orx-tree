use super::iter_ptr::DfsIterPtr;
use super::stack::Item;
use super::DepthFirstEnumeration;
use crate::helpers::Col;
use crate::memory::MemoryPolicy;
use crate::pinned_storage::PinnedStorage;
use crate::TreeVariant;
use alloc::vec::Vec;
use orx_self_or::SoM;
use orx_selfref_col::{NodePtr, Refs};

pub struct DfsIterInto<'a, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
{
    col: &'a mut Col<V, M, P>,
    root_ptr: NodePtr<V>,
    iter: DfsIterPtr<V, E, S>,
}

impl<'a, V, M, P, E, S> DfsIterInto<'a, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
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
    #[allow(clippy::type_complexity)]
    pub(crate) unsafe fn from(
        (col, iter, root_ptr): (&'a mut Col<V, M, P>, DfsIterPtr<V, E, S>, NodePtr<V>),
    ) -> Self {
        let node = unsafe { &mut *root_ptr.ptr_mut() };

        match node.prev().get() {
            Some(parent) => {
                let parent = unsafe { &mut *parent.ptr_mut() };
                let sibling_idx = parent.next_mut().remove(root_ptr.ptr() as usize);
                debug_assert!(sibling_idx.is_some());

                node.prev_mut().clear();
            }
            None => {
                // node_ptr points to the root node
                col.ends_mut().clear();
            }
        }

        Self {
            col,
            root_ptr,
            iter,
        }
    }

    fn take_element(&mut self, element: E::Item<NodePtr<V>>) -> E::Item<V::Item> {
        E::map_node_data(element, |ptr| {
            let col = unsafe { &mut *(self.col as *mut Col<V, M, P>) };
            col.close(&ptr)
        })
    }
}

impl<V, M, P, E, S> Iterator for DfsIterInto<'_, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
{
    type Item = E::Item<V::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|element| self.take_element(element))
    }
}

impl<V, M, P, E, S> Drop for DfsIterInto<'_, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: DepthFirstEnumeration,
    S: SoM<Vec<Item<V, E>>>,
{
    fn drop(&mut self) {
        while let Some(element) = self.iter.next() {
            self.take_element(element);
        }
        self.col.reclaim_from_closed_node(&self.root_ptr);
    }
}
