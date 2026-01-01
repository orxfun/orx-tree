use super::subtree_within::sealed::SubTreeWithinCore;
use crate::{
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant,
    iter::AncestorsIterPtr,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
};
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

pub struct ClonedSubTreeWithin<V: TreeVariant>
where
    V::Item: Clone,
{
    idx: NodeIdx<V>,
}

impl<V: TreeVariant> ClonedSubTreeWithin<V>
where
    V::Item: Clone,
{
    pub(crate) fn new(idx: NodeIdx<V>) -> Self {
        Self { idx }
    }
}

impl<V: TreeVariant> SubTreeWithinCore<V> for ClonedSubTreeWithin<V>
where
    V::Item: Clone,
{
    fn append_to_node_within_as_child<M, P, MO>(
        self,
        parent: &mut NodeMut<V, M, P, MO>,
        child_position: usize,
    ) -> NodeIdx<V>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
    {
        let ptr = self.idx.0.node_ptr();
        let iter_ptr = Dfs::<OverDepthPtr>::iter_ptr_with_owned_storage(ptr);

        let root_ptr = parent.root_ptr().expect("Non-empty tree has root.");
        let mut ancestors_of_parent = AncestorsIterPtr::new(root_ptr, parent.node_ptr());
        let subtree_is_ancestor_of_parent = ancestors_of_parent.any(|x| x == ptr);

        match subtree_is_ancestor_of_parent {
            false => append_subtree_as_child(parent, child_position, iter_ptr),
            true => {
                // SAFETY: when subtree is ancestor of the parent; we need to first
                // fix the nodes to be added (by collecting). Otherwise, we would
                // enter an infinite computation, recursively adding the subtree
                let iter_ptr = iter_ptr.collect::<Vec<_>>().into_iter();
                append_subtree_as_child(parent, child_position, iter_ptr);
            }
        }

        self.idx
    }
}

fn append_subtree_as_child<V, M, P, MO>(
    parent: &mut NodeMut<V, M, P, MO>,
    child_position: usize,
    iter_ptr: impl Iterator<Item = (usize, NodePtr<V>)>,
) where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
    V::Item: Clone,
{
    let subtree = iter_ptr.map(|(depth, ptr)| {
        (
            depth,
            unsafe { &*ptr.ptr() }
                .data()
                .expect("node is active")
                .clone(),
        )
    });
    parent.append_subtree_as_child(subtree, child_position);
}
