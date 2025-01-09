use super::{iter_ptr::BfsIterPtr, iter_ref::BfsIterRef, queue::Item, Bfs};
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverItem, traverser_core::TraverserCore, Over},
    MemoryPolicy, NodeRef, TreeVariant,
};
use alloc::collections::VecDeque;
use orx_self_or::SoM;

impl<O: Over> TraverserCore<O> for Bfs<O> {
    type Storage<V>
        = VecDeque<Item<V, O::Enumeration>>
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
        let iter = BfsIterPtr::<_, O::Enumeration, _>::from((storage, root));
        BfsIterRef::<'_, _, M, P, _, _, _>::from((node.col(), iter))
    }
}
