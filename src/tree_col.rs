use crate::{helpers::N, Node, NodeMut, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait TreeColCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn col(&self) -> &SelfRefCol<V, M, P>;

    // provided

    /// Returns the Node rooted at the node with the given `ptr`.
    fn ptr_to_tree_node(&self, ptr: NodePtr<V>) -> Node<V, M, P> {
        Node {
            col: self.col(),
            node_ptr: ptr,
        }
    }
}

// mut

pub trait TreeColMutCore<V, M, P>: TreeColCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn col_mut(&mut self) -> &mut SelfRefCol<V, M, P>;

    // provided

    /// Returns the collection node at the given `ptr`.
    fn ptr_to_node_mut<'a>(&'a mut self, ptr: NodePtr<V>) -> &'a mut N<V>
    where
        M: 'a,
        P: 'a,
    {
        self.col_mut().node_mut(&ptr)
    }

    /// Returns the NodeMut rooted at the node with the given `ptr`.
    fn ptr_to_tree_node_mut(&mut self, ptr: NodePtr<V>) -> NodeMut<V, M, P> {
        NodeMut {
            col: self.col_mut(),
            node_ptr: ptr,
        }
    }
}
