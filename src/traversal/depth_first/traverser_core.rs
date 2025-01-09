use super::{iter_ptr::DfsIterPtr, iter_ref::DfsIterRef, stack::Item, Dfs};
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverItem, traverser_core::TraverserCore, Over},
    MemoryPolicy, NodeRef, TreeVariant,
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
}
