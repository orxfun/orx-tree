use super::subtree::sealed::SubTreeCore;
use crate::{
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
    Dfs, MemoryPolicy, NodeRef, TreeVariant,
};
use core::marker::PhantomData;
use orx_selfref_col::NodePtr;

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

impl<'a, V, M, P, N> SubTreeCore<V> for ClonedSubTree<'a, V, M, P, N>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    N: NodeRef<'a, V, M, P>,
    V::Item: Clone,
{
    fn root_ptr(&self) -> NodePtr<V> {
        self.node.node_ptr().clone()
    }

    fn root_parent_ptr(&self) -> Option<NodePtr<V>> {
        let root = unsafe { &*self.node.node_ptr().ptr() };
        root.prev().get().cloned()
    }

    fn root_sibling_idx(&self) -> usize {
        self.node.sibling_idx()
    }

    fn create_subtree(&mut self) -> impl IntoIterator<Item = (usize, <V>::Item)> {
        let ptr = self.node.node_ptr().clone();
        let iter_ptr = Dfs::<OverDepthPtr>::iter_ptr_with_owned_storage(ptr);
        iter_ptr.map(|(depth, ptr)| {
            (
                depth,
                unsafe { &*ptr.ptr() }
                    .data()
                    .expect("node is active")
                    .clone(),
            )
        })
    }
}
