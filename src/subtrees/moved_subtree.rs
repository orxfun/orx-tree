use super::subtree::sealed::SubTreeCore;
use crate::{
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{traverser_core::TraverserCore, OverDepthData},
    Dfs, MemoryPolicy, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
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

    fn create_subtree(&mut self) -> impl IntoIterator<Item = (usize, <V>::Item)> {
        let node = unsafe { self.node.clone_node_mut() };
        Dfs::<OverDepthData>::into_iter_with_owned_storage(node)
    }
}
