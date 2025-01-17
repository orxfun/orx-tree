use super::subtree::SubTree;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthData, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, Tree, TreeVariant,
};

impl<V, M, P> SubTree<V::Item> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
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
