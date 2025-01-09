use super::{iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef, states::State, PostOrder};
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverItem, traverser_core::TraverserCore, Over},
    MemoryPolicy, NodeRef, TreeVariant,
};
use alloc::vec::Vec;
use orx_self_or::SoM;

impl<O: Over> TraverserCore<O> for PostOrder<O> {
    type Storage<V>
        = Vec<State<V>>
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
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((storage, root));
        PostOrderIterRef::from((node.col(), iter_ptr))
    }
}
