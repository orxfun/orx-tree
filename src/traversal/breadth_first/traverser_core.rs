use super::{
    Bfs, into_iter::BfsIterInto, iter_mut::BfsIterMut, iter_ptr::BfsIterPtr, iter_ref::BfsIterRef,
    queue::Item,
};
use crate::{
    MemoryPolicy, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        Over, OverMut,
        breadth_first::into_iter_filtered::BfsIterIntoFiltered,
        enumeration::Enumeration,
        over::OverItem,
        over_mut::{OverItemInto, OverItemMut},
        traverser_core::TraverserCore,
    },
};
use alloc::collections::VecDeque;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

impl<O: Over> TraverserCore<O> for Bfs<O> {
    type Storage<V>
        = VecDeque<Item<V, O::Enumeration>>
    where
        V: TreeVariant;

    fn storage_mut<V: TreeVariant>(&mut self) -> &mut Self::Storage<V> {
        self.queue.for_variant::<V>()
    }

    fn iter_ptr_with_storage<'a, V>(
        node_ptr: NodePtr<V>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = <<O as Over>::Enumeration as Enumeration>::Item<NodePtr<V>>>
    where
        V: TreeVariant + 'a,
    {
        BfsIterPtr::<_, O::Enumeration, _>::from((storage, node_ptr))
    }

    fn iter_with_storage<'t, 'a, V, M, P>(
        node: &impl NodeRef<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>> + 't,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        Self::Storage<V>: 't,
        'a: 't,
    {
        let root = node.node_ptr();
        let iter = BfsIterPtr::<_, O::Enumeration, _>::from((storage, root));
        BfsIterRef::<'_, _, M, P, _, _, _>::from((node.col(), iter))
    }

    fn iter<'t, 'a, V, M, P>(
        &'t mut self,
        node: &'t impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        'a: 't,
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
        let root = node_mut.node_ptr();
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
        O: Over,
    {
        let (col, root) = node_mut.into_inner();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((storage, root));
        unsafe { BfsIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }

    fn into_iter_with_storage_filtered<'a, V, M, P, MO, F>(
        node_mut: NodeMut<'a, V, M, P, MO>,
        storage: impl SoM<Self::Storage<V>>,
        filter: F,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: Over,
        F: Fn(&<<O as Over>::Enumeration as Enumeration>::Item<NodePtr<V>>) -> bool,
    {
        let (col, root) = node_mut.into_inner();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((storage, root));
        unsafe { BfsIterIntoFiltered::<V, M, P, _, _, _>::from((col, iter_ptr, root), filter) }
    }
}
