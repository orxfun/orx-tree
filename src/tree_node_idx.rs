use crate::{pinned_storage::PinnedStorage, Node, NodeMut, Tree, TreeMemoryPolicy, TreeVariant};
use orx_selfref_col::{MemoryState, NodePtr};

pub struct NodeIdx<V: TreeVariant>(orx_selfref_col::NodeIdx<V>);

impl<V: TreeVariant> Clone for NodeIdx<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: TreeVariant> PartialEq for NodeIdx<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V: TreeVariant> NodeIdx<V> {
    pub(crate) fn new(state: MemoryState, node_ptr: &NodePtr<V>) -> Self {
        Self(orx_selfref_col::NodeIdx::new(state, node_ptr))
    }

    #[inline(always)]
    pub fn is_valid_for<M, P>(&self, tree: &Tree<V, M, P>) -> bool
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        self.0.is_valid_for(&tree.0)
    }

    pub fn node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.0.is_valid_for(&tree.0));
        Node::new(&tree.0, self.0.node_ptr())
    }

    pub fn node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.0.is_valid_for(&tree.0));
        NodeMut::new(&mut tree.0, self.0.node_ptr())
    }

    pub fn get_node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Option<Node<'a, V, M, P>>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        self.0
            .is_valid_for(&tree.0)
            .then(|| Node::new(&tree.0, self.0.node_ptr()))
    }

    pub fn get_node_mut<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> Option<NodeMut<'a, V, M, P>>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        self.0
            .is_valid_for(&tree.0)
            .then(|| NodeMut::new(&mut tree.0, self.0.node_ptr()))
    }

    pub unsafe fn node_unchecked<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        Node::new(&tree.0, self.0.node_ptr())
    }

    pub unsafe fn node_mut_unchecked<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> NodeMut<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        NodeMut::new(&mut tree.0, self.0.node_ptr())
    }
}
