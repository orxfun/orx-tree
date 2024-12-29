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

impl<V, M, P> TreeColCore<V, M, P> for NodeMut<'_, V, M, P>
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

impl<V, M, P> TreeColMutCore<V, M, P> for NodeMut<'_, V, M, P>
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

impl<V, M, P> NodeRefCore<V, M, P> for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
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
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// *root.data_mut() = 10;
    /// assert_eq!(root.data(), &10);
    ///
    /// let a = root.push(1);
    /// let mut node = tree.node_mut(&a).unwrap();
    /// *node.data_mut() += 10;
    /// assert_eq!(node.data(), &11);
    /// ```
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    pub fn data_mut(&mut self) -> &mut V::Item {
        self.node_mut()
            .data_mut()
            .expect("node holding a tree reference must be active")
    }

    /// Pushes the node with the given `value` as a child of this node.
    ///
    /// Returns the index of the created child node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<char>::new('a');
    ///
    /// let mut node = tree.root_mut().unwrap();
    /// let b = node.push('b'); // b is the index of the node
    /// let c = node.push('c');
    ///
    /// let node_b = tree.node(&b).unwrap(); // access the node via idx
    /// assert_eq!(node_b.data(), &'b');
    ///
    /// let mut node_c = tree.node_mut(&c).unwrap();
    /// let d = node_c.push('d');
    /// node_c.extend(['e', 'f']);
    /// assert_eq!(node_c.num_children(), 3);
    /// ```
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

    /// Pushes nodes with given `values` as children of this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<char>::new('a');
    ///
    /// let mut node = tree.root_mut().unwrap();
    /// let b = node.push('b'); // b is the index of the node
    /// node.extend(['c', 'd', 'e']);
    ///
    /// assert_eq!(node.num_children(), 4);
    /// ```
    pub fn extend<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = V::Item>,
    {
        for x in values.into_iter() {
            _ = self.push(x);
        }
    }

    /// Pushes nodes with the given `values` as children of this node.
    ///
    /// Returns the indices of the created child nodes.
    ///
    /// Note that this method returns a lazy iterator.
    /// Unless the iterator is consumed, the nodes will not be pushed to the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<char>::new('a');
    ///
    /// let mut node = tree.root_mut().unwrap();
    /// let b = node.push('b'); // b is the index of the node
    /// let cde = node.extend_get_indices(['c', 'd', 'e']).collect::<Vec<_>>();
    ///
    /// assert_eq!(node.num_children(), 4);
    ///
    /// let node_d = tree.node(&cde[1]).unwrap();
    /// assert_eq!(node_d.data(), &'d');
    /// ```
    pub fn extend_get_indices<'b, I>(
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

    fn node_mut(&mut self) -> &mut N<V> {
        unsafe { &mut *self.node_ptr().ptr_mut() }
    }
}
