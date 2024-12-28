use crate::{helpers::N, Node, NodeMut, Tree, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodeIdx};

pub trait AsTreeNode<V>
where
    V: TreeVariant,
{
    fn as_node<'a, M, P>(&'a self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>;

    fn as_node_mut<'a, M, P>(&'a mut self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>;
}

impl<V> AsTreeNode<V> for NodeIdx<V>
where
    V: TreeVariant,
{
    fn as_node<'a, M, P>(&'a self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
    {
        Node {
            col: &tree.0,
            node_ptr: self.node_ptr(),
        }
    }

    fn as_node_mut<'a, M, P>(&'a mut self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
    {
        NodeMut {
            col: &mut tree.0,
            node_ptr: self.node_ptr(),
        }
    }
}
