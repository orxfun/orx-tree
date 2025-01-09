use super::{iter_mut::DfsIterMut, iter_ptr::DfsIterPtr, iter_ref::DfsIterRef, stack::Item, Dfs};
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::OverItem, over_mut::OverItemMut, traverser_core::TraverserCore, Over, OverMut,
    },
    MemoryPolicy, NodeMut, NodeRef, TreeVariant,
};
use alloc::vec::Vec;
use orx_self_or::SoM;

impl<O: Over> TraverserCore<O> for Dfs<O> {
    type Storage<V>
        = Vec<Item<V, O::Enumeration>>
    where
        V: TreeVariant;

    fn iter_with_storage<'a, V, M, P>(
        node: &'a impl NodeRef<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        let root = node.node_ptr().clone();
        let iter = DfsIterPtr::<_, O::Enumeration, _>::from((storage, root));
        DfsIterRef::<'_, _, M, P, _, _, _>::from((node.col(), iter))
    }

    fn iter_mut_with_storage<'a, V, M, P>(
        node_mut: &'a mut NodeMut<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut,
    {
        let root = node_mut.node_ptr().clone();
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((storage, root));
        unsafe { DfsIterMut::from((node_mut.col(), iter_ptr)) }
    }
}
