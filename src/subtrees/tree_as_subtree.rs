use super::subtree::sealed::SubTreeCore;
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthData, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, Tree, TreeVariant,
};
use orx_selfref_col::NodePtr;

impl<V, M, P> SubTreeCore<V> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn root_ptr(&self) -> NodePtr<V> {
        self.root().node_ptr().clone()
    }

    fn root_parent_ptr(&self) -> Option<NodePtr<V>> {
        None
    }

    fn root_sibling_idx(&self) -> usize {
        0
    }

    // TODO: convert to O(1) by using SplitVec Recursive properties
    fn append_to_node_as_child<V2, M2, P2, MO>(
        mut self,
        parent: &mut NodeMut<V2, M2, P2, MO>,
        child_idx: usize,
    ) -> NodeIdx<V2>
    where
        V2: TreeVariant<Item = V::Item>,
        M2: MemoryPolicy,
        P2: PinnedStorage,
        MO: crate::NodeMutOrientation,
    {
        let root = self.root_mut();
        let subtree = Dfs::<OverDepthData>::into_iter_with_owned_storage(root);
        parent.append_subtree_as_child(subtree, child_idx)
    }
}
