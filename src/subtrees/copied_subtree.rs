use super::subtree::sealed::SubTreeCore;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
};
use core::marker::PhantomData;
use orx_selfref_col::NodePtr;

pub struct CopiedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Copy,
{
    node: N,
    phantom: PhantomData<&'a (V, M, P)>,
}

impl<'a, V, M, P, N> CopiedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Copy,
{
    pub(crate) fn new(node: N) -> Self {
        Self {
            node,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, N> SubTreeCore<V> for CopiedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Copy,
{
    fn root_ptr(&self) -> NodePtr<V> {
        self.node.node_ptr().clone()
    }

    fn append_to_node_as_child<V2, M2, P2, MO>(
        self,
        parent: &mut NodeMut<V2, M2, P2, MO>,
        child_position: usize,
    ) -> NodeIdx<V2>
    where
        V2: TreeVariant<Item = V::Item>,
        M2: MemoryPolicy,
        P2: PinnedStorage,
        MO: NodeMutOrientation,
    {
        let ptr = self.node.node_ptr().clone();
        let iter_ptr = Dfs::<OverDepthPtr>::iter_ptr_with_owned_storage(ptr);
        let subtree = iter_ptr.map(|(depth, ptr)| {
            (
                depth,
                *unsafe { &*ptr.ptr() }.data().expect("node is active"),
            )
        });

        parent.append_subtree_as_child(subtree, child_position)
    }
}
