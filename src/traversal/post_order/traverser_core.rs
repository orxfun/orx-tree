use super::{
    into_iter::PostOrderIterInto, iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr,
    iter_ref::PostOrderIterRef, states::State, PostOrder,
};
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        enumeration::Enumeration,
        over::OverItem,
        over_mut::{OverItemInto, OverItemMut},
        traverser_core::TraverserCore,
        Over, OverMut,
    },
    MemoryPolicy, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
};
use alloc::vec::Vec;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

impl<O: Over> TraverserCore<O> for PostOrder<O> {
    type Storage<V>
        = Vec<State<V>>
    where
        V: TreeVariant;

    fn iter_ptr_with_storage<'a, V>(
        node_ptr: NodePtr<V>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = <<O as Over>::Enumeration as Enumeration>::Item<NodePtr<V>>>
    where
        V: TreeVariant + 'a,
    {
        PostOrderIterPtr::<_, O::Enumeration, _>::from((storage, node_ptr))
    }

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
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((storage, root));
        PostOrderIterRef::from((node.col(), iter_ptr))
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
        let states = self.states.for_variant::<V>();
        Self::iter_with_storage(node, states)
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
        let states = self.states.for_variant::<V>();
        Self::iter_mut_with_storage(node_mut, states)
    }

    fn into_iter<'a, V, M, P, MO>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = crate::traversal::over_mut::OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
    {
        let states = self.states.for_variant::<V>();
        Self::into_iter_with_storage(node_mut, states)
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
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((storage, root));
        unsafe { PostOrderIterMut::from((node_mut.col(), iter_ptr)) }
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
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((storage, root.clone()));
        unsafe { PostOrderIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}
