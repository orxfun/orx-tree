use crate::{
    helpers::N, memory::TreeMemoryPolicy, node_ref::NodeRefCore, Node, NodeMut, TreeVariant,
};
use orx_pinned_vec::PinnedVec;

impl<V, M, P> PartialEq for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<'a, V, M, P> PartialEq<NodeMut<'a, V, M, P>> for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &NodeMut<'a, V, M, P>) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<V, M, P> PartialEq for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Self) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}

impl<'a, V, M, P> PartialEq<Node<'a, V, M, P>> for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: TreeMemoryPolicy,
    P: PinnedVec<N<V>>,
{
    fn eq(&self, other: &Node<'a, V, M, P>) -> bool {
        self.node_ptr() == other.node_ptr()
    }
}
