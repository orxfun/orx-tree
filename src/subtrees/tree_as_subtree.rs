use super::subtree::SubTree;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, Tree, TreeVariant,
};

impl<V, M, P> SubTree<V::Item> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn dfs_iter(&mut self) -> impl IntoIterator<Item = (usize, V::Item)> {
        Dfs::<OverDepthPtr>::into_iter_with_owned_storage(self.root_mut())
    }
}
