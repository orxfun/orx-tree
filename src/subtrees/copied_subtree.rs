use super::subtree::sealed::SubTreeCore;
use crate::{
    Dfs, MemoryPolicy, NodeRef, TreeVariant,
    pinned_storage::PinnedStorage,
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
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
        self.node.node_ptr()
    }

    fn root_parent_ptr(&self) -> Option<NodePtr<V>> {
        let root = unsafe { &*self.node.node_ptr().ptr() };
        root.prev().get()
    }

    fn root_sibling_idx(&self) -> usize {
        self.node.sibling_idx()
    }

    fn create_subtree(&mut self) -> impl IntoIterator<Item = (usize, <V>::Item)> {
        let ptr = self.node.node_ptr();
        let iter_ptr = Dfs::<OverDepthPtr>::iter_ptr_with_owned_storage(ptr);
        iter_ptr.map(|(depth, ptr)| {
            (
                depth,
                *unsafe { &*ptr.ptr() }.data().expect("node is active"),
            )
        })
    }
}
