use orx_self_or::SoM;

use super::{
    over::{Over, OverItem},
    over_mut::{OverItemInto, OverItemMut},
    OverData, OverMut,
};
use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage, NodeMut, NodeRef, TreeVariant};

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
        P: PinnedStorage,
    {
        core::iter::empty()
    }

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
}
