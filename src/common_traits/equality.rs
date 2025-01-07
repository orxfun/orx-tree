use crate::{
    memory::TreeMemoryPolicy, node_ref::NodeRefCore, pinned_storage::PinnedStorage, Node, NodeMut,
    TreeVariant,
};

impl<V, M, P> PartialEq for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedStorage,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<'a, V, M, P> PartialEq<NodeMut<'a, V, M, P>> for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedStorage,
{
    fn eq(&self, other: &NodeMut<'a, V, M, P>) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<V, M, P> PartialEq for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedStorage,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<'a, V, M, P> PartialEq<Node<'a, V, M, P>> for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedStorage,
{
    fn eq(&self, other: &Node<'a, V, M, P>) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}
