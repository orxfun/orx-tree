use super::subtree_within::sealed::SubTreeWithinCore;
use crate::{
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant,
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
};

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

        self.idx
    }
}
