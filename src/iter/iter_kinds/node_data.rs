use crate::{helpers::N, Node, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub enum Never {}

// traits

/// Part of the iterator item that is obtained from the tree node.
pub trait NodeData {
    /// Type of the value extracted from the node.
    type Value<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    /// Type of the mutable value extracted from the node.
    type ValueMut<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    /// Gets the value from the node.
    fn value<'a, V, M, P>(
        col: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> Self::Value<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    /// Gets the mutable value from the node.
    fn value_mut<'a, V, M, P>(
        col: &'a mut SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> Self::ValueMut<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

// impl

/// Returns the node pointer.
pub struct NodeDataPtr;

impl NodeData for NodeDataPtr {
    type Value<'a, V, M, P>
        = NodePtr<V>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type ValueMut<'a, V, M, P>
        = Never
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    #[inline(always)]
    fn value<'a, V, M, P>(
        _: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> Self::Value<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        node_ptr
    }

    fn value_mut<'a, V, M, P>(
        _: &'a mut SelfRefCol<V, M, P>,
        _: NodePtr<V>,
    ) -> Self::ValueMut<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        unreachable!("cannot iterate over mutable nodes")
    }
}

/// Returns the entire node.
pub struct NodeDataNode;

impl NodeData for NodeDataNode {
    type Value<'a, V, M, P>
        = Node<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type ValueMut<'a, V, M, P>
        = Never
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    #[inline(always)]
    fn value<'a, V, M, P>(
        col: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> Self::Value<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        Node::new(col, node_ptr)
    }

    fn value_mut<'a, V, M, P>(
        _: &'a mut SelfRefCol<V, M, P>,
        _: NodePtr<V>,
    ) -> Self::ValueMut<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        unreachable!("cannot iterate over mutable nodes")
    }
}

/// Returns a reference to the node.
pub struct NodeDataData;

impl NodeData for NodeDataData {
    type Value<'a, V, M, P>
        = &'a V::Item
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type ValueMut<'a, V, M, P>
        = &'a mut V::Item
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    #[inline(always)]
    fn value<'a, V, M, P>(
        _: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> Self::Value<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        let node = unsafe { &*node_ptr.ptr() };
        node.data().expect("active tree node cannot be closed")
    }

    #[inline(always)]
    fn value_mut<'a, V, M, P>(
        _: &'a mut SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> Self::ValueMut<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        let node = unsafe { &mut *node_ptr.ptr_mut() };
        node.data_mut().expect("active tree node cannot be closed")
    }
}
