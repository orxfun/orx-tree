use super::subtree::sealed::SubTreeCore;
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthData, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, Tree, TreeVariant,
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

    fn into_subtree(&mut self) -> impl IntoIterator<Item = (usize, <V>::Item)> {
        let root = self.root_mut();
        Dfs::<OverDepthData>::into_iter_with_owned_storage(root)
    }
}
