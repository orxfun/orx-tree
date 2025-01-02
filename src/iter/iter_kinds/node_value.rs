use crate::{helpers::N, Node, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub enum Never {}

// traits

/// Part of the iterator item that is obtained from the tree node.
pub trait NodeValue<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    /// Type of the value extracted from the node.
    type Value;

    /// Type of the mutable value extracted from the node.
    type ValueMut;

    /// Gets the value from the node.
    fn value(col: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value;

    /// Gets the mutable value from the node.
    fn value_mut(col: &'a SelfRefCol<V, M, P>, node: &'a mut N<V>) -> Self::ValueMut;
}

// impl

/// Returns the node pointer.
pub struct NodeValuePtr;

impl<'a, V, M, P> NodeValue<'a, V, M, P> for NodeValuePtr
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = NodePtr<V>;

    type ValueMut = Never;

    #[inline(always)]
    fn value(_: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        NodePtr::new(node)
    }

    fn value_mut(_: &'a SelfRefCol<V, M, P>, _: &'a mut N<V>) -> Self::ValueMut {
        unreachable!("cannot iterate over mutable nodes")
    }
}

/// Returns the entire node.
pub struct NodeValueNode;

impl<'a, V, M, P> NodeValue<'a, V, M, P> for NodeValueNode
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = Node<'a, V, M, P>;

    type ValueMut = Never;

    #[inline(always)]
    fn value(col: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        Node::new(col, NodePtr::new(node as *const N<V>))
    }

    fn value_mut(_: &'a SelfRefCol<V, M, P>, _: &'a mut N<V>) -> Self::ValueMut {
        unreachable!("cannot iterate over mutable nodes")
    }
}

/// Returns a reference to the node.
pub struct NodeValueData;

impl<'a, V, M, P> NodeValue<'a, V, M, P> for NodeValueData
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = &'a V::Item;

    type ValueMut = &'a mut V::Item;

    #[inline(always)]
    fn value(_: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        node.data().expect("active tree node cannot be closed")
    }

    #[inline(always)]
    fn value_mut(_: &'a SelfRefCol<V, M, P>, node: &'a mut N<V>) -> Self::ValueMut {
        node.data_mut().expect("active tree node cannot be closed")
    }
}
