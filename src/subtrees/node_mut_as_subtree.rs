use super::subtree::SubTree;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, TreeVariant,
};

impl<'a, V, M, P> SubTree<V::Item> for NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn append_to_node_as_child<V2, M2, P2, MO>(
        self,
        parent: &mut NodeMut<V2, M2, P2, MO>,
    ) -> NodeIdx<V2>
    where
        V2: TreeVariant<Item = V::Item>,
        M2: MemoryPolicy,
        P2: PinnedStorage,
        MO: crate::NodeMutOrientation,
    {
        let subtree = Dfs::<OverDepthPtr>::into_iter_with_owned_storage(self);
        parent.append_subtree_as_child(subtree)
    }
}
