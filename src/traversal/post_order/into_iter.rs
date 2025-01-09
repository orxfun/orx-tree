use super::iter_ptr::PostOrderIterPtr;
use super::post_enumeration::PostOrderEnumeration;
use super::states::States;
use crate::helpers::Col;
use crate::memory::MemoryPolicy;
use crate::pinned_storage::PinnedStorage;
use crate::TreeVariant;
use orx_self_or::SoM;
use orx_selfref_col::{NodePtr, Refs};

pub struct PostOrderIterInto<'a, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
{
    col: &'a mut Col<V, M, P>,
    iter: PostOrderIterPtr<V, E, S>,
}

impl<'a, V, M, P, E, S> PostOrderIterInto<'a, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
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
    pub(crate) unsafe fn from(
        (col, iter, node_ptr): (&'a mut Col<V, M, P>, PostOrderIterPtr<V, E, S>, NodePtr<V>),
    ) -> Self {
        let node = unsafe { &mut *node_ptr.ptr_mut() };

        match node.prev().get() {
            Some(parent) => {
                let parent = unsafe { &mut *parent.ptr_mut() };
                let sibling_idx = parent.next_mut().remove(node_ptr.ptr() as usize);
                debug_assert!(sibling_idx.is_some());

                node.prev_mut().clear();
            }
            None => {
                // node_ptr points to the root node
                col.ends_mut().clear();
            }
        }

        Self { col, iter }
    }
}

impl<'a, V, M, P, E, S> Iterator for PostOrderIterInto<'a, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
{
    type Item = E::Item<V::Item>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|element| {
            E::map_node_data(element, |ptr| {
                let col = unsafe { &mut *(self.col as *mut Col<V, M, P>) };
                col.close(&ptr)
            })
        })
    }
}

impl<'a, V, M, P, E, S> Drop for PostOrderIterInto<'a, V, M, P, E, S>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    E: PostOrderEnumeration,
    S: SoM<States<V>>,
{
    fn drop(&mut self) {
        // to be done later
    }
}
