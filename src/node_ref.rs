use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

pub trait NodeRefCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn node_ptr(&self) -> &NodePtr<V>;

    fn node(&self) -> &N<V> {
        unsafe { &*self.node_ptr().ptr() }
    }
}

impl<V, M, P, X> NodeRef<V, M, P> for X
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    X: NodeRefCore<V, M, P>,
{
}

/// Reference to a tree node.
pub trait NodeRef<V, M, P>: NodeRefCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Returns a reference to the data of the node.
    fn data<'a>(&'a self) -> &'a V::Item
    where
        V: 'a,
    {
        self.node()
            .data()
            .expect("node holding a tree reference must be active")
    }

    /// Returns the number of child nodes of this node.
    fn num_children(&self) -> usize {
        self.node().next().num_children()
    }
}
