use super::subtree::sealed::SubTreeCore;
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{traverser_core::TraverserCore, OverDepthData},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
};
use orx_selfref_col::NodePtr;

pub struct MovedSubTree<'a, V, M, P, MO>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    node: NodeMut<'a, V, M, P, MO>,
}

impl<'a, V, M, P, MO> MovedSubTree<'a, V, M, P, MO>
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

impl<'a, V, M, P, MO> SubTreeCore<V> for MovedSubTree<'a, V, M, P, MO>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    fn root_ptr(&self) -> NodePtr<V> {
        self.node.node_ptr().clone()
    }

    fn root_parent_ptr(&self) -> Option<NodePtr<V>> {
        self.node.parent_ptr()
    }

    fn root_sibling_idx(&self) -> usize {
        self.node.sibling_idx()
    }

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
