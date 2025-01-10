use super::{
    into_iter::BfsIterInto, iter_mut::BfsIterMut, iter_ptr::BfsIterPtr, iter_ref::BfsIterRef,
    queue::Item, Bfs,
};
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::OverItem,
        over_mut::{OverItemInto, OverItemMut},
        traverser_core::TraverserCore,
        Over, OverMut,
    },
    MemoryPolicy, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
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

    fn iter<'a, V, M, P>(
        &'a mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        let queue = self.queue.for_variant::<V>();
        Self::iter_with_storage(node, queue)
    }

    fn iter_mut<'a, V, M, P, MO>(
        &'a mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
    {
        let queue = self.queue.for_variant::<V>();
        Self::iter_mut_with_storage(node_mut, queue)
    }

    fn into_iter<'a, V, M, P, MO>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
    {
        let queue = self.queue.for_variant::<V>();
        Self::into_iter_with_storage(node_mut, queue)
    }

    fn iter_mut_with_storage<'a, V, M, P, MO>(
        node_mut: &'a mut NodeMut<'a, V, M, P, MO>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
    {
        let root = node_mut.node_ptr().clone();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((storage, root));
        unsafe { BfsIterMut::from((node_mut.col(), iter_ptr)) }
    }

    fn into_iter_with_storage<'a, V, M, P, MO>(
        node_mut: NodeMut<'a, V, M, P, MO>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
    {
        let (col, root) = node_mut.into_inner();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((storage, root.clone()));
        unsafe { BfsIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}
