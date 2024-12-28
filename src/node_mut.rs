use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    tree::{DefaultMemory, DefaultPinVec},
    tree_col::{TreeColCore, TreeColMutCore},
    tree_variant::RefsChildren,
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodeIdx, NodePtr, SelfRefCol};

/// A node of the tree, which in turn is a tree.
pub struct NodeMut<'a, V, M = DefaultMemory<V>, P = DefaultPinVec<V>>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) col: &'a mut SelfRefCol<V, M, P>,
    pub(crate) node_ptr: NodePtr<V>,
}

impl<'a, V, M, P> TreeColCore<V, M, P> for NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn col(&self) -> &SelfRefCol<V, M, P> {
        self.col
    }
}

impl<'a, V, M, P> TreeColMutCore<V, M, P> for NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn col_mut(&mut self) -> &mut SelfRefCol<V, M, P> {
        self.col
    }
}

impl<'a, V, M, P> NodeRefCore<V, M, P> for NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.node_ptr
    }
}

impl<'a, V, M, P> NodeMut<'a, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Returns a mutable reference to data of the root node.
    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut V::Item {
        self.node_mut()
            .data_mut()
            .expect("node holding a tree reference must be active")
    }

    /// Pushes the node with the given `value` as a child of this node.
    ///
    /// Returns the index of the created child node.
    pub fn push(&mut self, value: V::Item) -> NodeIdx<V> {
        let parent_ptr = self.node_ptr.clone();

        let child_idx = self.col_mut().push_get_idx(value);
        let child_ptr = child_idx.node_ptr();

        let child = self.ptr_to_node_mut(child_ptr.clone());
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = self.ptr_to_node_mut(parent_ptr);
        parent.next_mut().push(child_ptr);

        child_idx
    }

    /// Pushes the node with the given `values` as children of this node.
    ///
    /// Returns the indices of the created child nodes.
    pub fn extend<'b, I>(
        &'b mut self,
        values: I,
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P>
    where
        I: IntoIterator<Item = V::Item>,
        I::IntoIter: 'b,
    {
        values.into_iter().map(|x| self.push(x))
    }

    // helpers

    fn node_mut(&self) -> &mut N<V> {
        unsafe { &mut *self.node_ptr().ptr_mut() }
    }
}
