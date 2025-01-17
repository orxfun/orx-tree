use super::subtree::SubTree;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{traverser_core::TraverserCore, OverDepthData},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant,
};

pub struct NodeMutAsSubTree<'a, V, M, P, MO>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    node: NodeMut<'a, V, M, P, MO>,
}

impl<'a, V, M, P, MO> NodeMutAsSubTree<'a, V, M, P, MO>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    pub(crate) fn new(node: NodeMut<'a, V, M, P, MO>) -> Self {
        Self { node }
    }
}

impl<'a, V, M, P, MO> SubTree<V::Item> for NodeMutAsSubTree<'a, V, M, P, MO>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    fn append_to_node_as_child<V2, M2, P2, MO2>(
        self,
        parent: &mut NodeMut<V2, M2, P2, MO2>,
        child_idx: usize,
    ) -> NodeIdx<V2>
    where
        V2: TreeVariant<Item = V::Item>,
        M2: MemoryPolicy,
        P2: PinnedStorage,
        MO2: NodeMutOrientation,
    {
        let subtree = Dfs::<OverDepthData>::into_iter_with_owned_storage(self.node);
        parent.append_subtree_as_child(subtree, child_idx)
    }
}
