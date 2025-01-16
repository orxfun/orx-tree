use super::SubTree;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, NodeRef, TreeVariant,
};
use core::marker::PhantomData;

pub struct ClonedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Clone,
{
    node: N,
    phantom: PhantomData<&'a (V, M, P)>,
}

impl<'a, V, M, P, N> ClonedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Clone,
{
    pub(crate) fn new(node: N) -> Self {
        Self {
            node,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P, N> SubTree<V::Item> for ClonedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Clone,
{
    fn append_to_node_as_child<V2, M2, P2, MO>(
        self,
        parent: &mut NodeMut<V2, M2, P2, MO>,
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
                unsafe { &*ptr.ptr() }
                    .data()
                    .expect("node is active")
                    .clone(),
            )
        });

        parent.append_subtree_as_child(subtree)
    }
}
