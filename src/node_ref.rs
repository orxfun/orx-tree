use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

pub trait NodeRefCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn node_ptr(&self) -> &NodePtr<V>;
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
        unsafe { &*self.node_ptr().ptr() }
            .data()
            .expect("node holding a tree reference cannot be closed")
    }
}
