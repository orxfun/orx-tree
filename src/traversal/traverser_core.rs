use super::{
    over::{Over, OverItem},
    over_mut::{OverItemInto, OverItemMut},
    OverData, OverMut,
};
use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage, NodeMut, NodeRef, TreeVariant};
use orx_self_or::SoM;

pub trait TraverserCore<O = OverData>: Sized
where
    O: Over,
{
    type Storage<V>: Default
    where
        V: TreeVariant;

    fn iter_with_storage<'a, V, M, P>(
        node: &'a impl NodeRef<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage;

    fn iter_mut_with_storage<'a, V, M, P>(
        node_mut: &'a mut NodeMut<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut;

    fn into_iter_with_storage<'a, V, M, P>(
        node_mut: NodeMut<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut;

    // provided

    fn iter_with_owned_storage<'a, V, M, P>(
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        Self::iter_with_storage(node, Self::Storage::default())
    }

    fn iter_mut_with_owned_storage<'a, V, M, P>(
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut,
    {
        Self::iter_mut_with_storage(node_mut, Self::Storage::default())
    }

    fn into_iter_with_owned_storage<'a, V, M, P>(
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut,
    {
        Self::into_iter_with_storage(node_mut, Self::Storage::default())
    }
}
