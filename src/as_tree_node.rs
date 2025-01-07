use crate::{
    memory::TreeMemoryPolicy, pinned_storage::PinnedStorage, Node, NodeMut, Tree, TreeVariant,
};
use orx_selfref_col::NodeIdx;

pub trait AsTreeNode<V>
where
    V: TreeVariant,
{
    fn is_valid_for<M, P>(&self, tree: &Tree<V, M, P>) -> bool
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;

    fn node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;

    fn node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;

    fn get_node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Option<Node<'a, V, M, P>>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;

    fn get_node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> Option<NodeMut<'a, V, M, P>>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;

    /// # Safety
    ///
    /// TODO
    unsafe fn node_unchecked<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;

    /// # Safety
    ///
    /// TODO
    unsafe fn node_mut_unchecked<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> NodeMut<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage;
}

impl<V> AsTreeNode<V> for NodeIdx<V>
where
    V: TreeVariant,
{
    #[inline(always)]
    fn is_valid_for<M, P>(&self, tree: &Tree<V, M, P>) -> bool
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        self.is_valid_for(&tree.0)
    }

    fn node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.is_valid_for(&tree.0));
        Node::new(&tree.0, self.node_ptr())
    }

    fn node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.is_valid_for(&tree.0));
        NodeMut::new(&mut tree.0, self.node_ptr())
    }

    fn get_node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Option<Node<'a, V, M, P>>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        self.is_valid_for(&tree.0)
            .then(|| Node::new(&tree.0, self.node_ptr()))
    }

    fn get_node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> Option<NodeMut<'a, V, M, P>>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        self.is_valid_for(&tree.0)
            .then(|| NodeMut::new(&mut tree.0, self.node_ptr()))
    }

    unsafe fn node_unchecked<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        Node::new(&tree.0, self.node_ptr())
    }

    unsafe fn node_mut_unchecked<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> NodeMut<'a, V, M, P>
    where
        M: TreeMemoryPolicy,
        P: PinnedStorage,
    {
        NodeMut::new(&mut tree.0, self.node_ptr())
    }
}
