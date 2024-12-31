use crate::{helpers::N, Node, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Part of the iterator item that is obtained from the tree node.
pub trait ValueFromNode<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    /// Type of the value extracted from the node.
    type Value;

    /// Gets the value from the node.
    fn value(col: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value;
}

/// Returns the entire node.
pub struct NodeFromNode;

impl<'a, V, M, P> ValueFromNode<'a, V, M, P> for NodeFromNode
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = Node<'a, V, M, P>;

    #[inline(always)]
    fn value(col: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        Node::new(col, NodePtr::new(node as *const N<V>))
    }
}

/// Returns a reference to the node.
pub struct DataFromNode;

impl<'a, V, M, P> ValueFromNode<'a, V, M, P> for DataFromNode
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = &'a V::Item;

    #[inline(always)]
    fn value(_: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        node.data().expect("active tree node cannot be closed")
    }
}
