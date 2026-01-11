use super::subtree_within::sealed::SubTreeWithinCore;
use crate::{
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant,
    iter::AncestorsIterPtr,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    subtrees_within::cloned_subtree::append_subtree_as_child,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
};
use alloc::vec::Vec;

pub struct CopiedSubTreeWithin<V: TreeVariant>
where
    V::Item: Copy,
{
    idx: NodeIdx<V>,
}

impl<V: TreeVariant> CopiedSubTreeWithin<V>
where
    V::Item: Copy,
{
    pub(crate) fn new(idx: NodeIdx<V>) -> Self {
        Self { idx }
    }
}

impl<V: TreeVariant> SubTreeWithinCore<V> for CopiedSubTreeWithin<V>
where
    V::Item: Copy,
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
