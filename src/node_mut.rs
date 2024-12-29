use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    tree::{DefaultMemory, DefaultPinVec},
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
    col: &'a mut SelfRefCol<V, M, P>,
    node_ptr: NodePtr<V>,
}

impl<V, M, P> NodeRefCore<V, M, P> for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn col(&self) -> &SelfRefCol<V, M, P> {
        self.col
    }

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

        let child_idx = self.col.push_get_idx(value);
        let child_ptr = child_idx.node_ptr();

        let child = self.col.node_mut(&child_ptr);
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = self.col.node_mut(&parent_ptr);
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

    /// Consumes this mutable node and returns the mutable node of the `child-index`-th child;
    /// returns None if the child index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the tree:
    /// // r
    /// // |-- a
    /// //     |-- c, d, e
    /// // |-- b
    /// //     |-- f, g
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.extend(['a', 'b']);
    ///
    /// let mut a = root.child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// // use child to access lower level nodes
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let data: Vec<_> = root.children().map(|x| *x.data()).collect();
    /// assert_eq!(data, ['a', 'b']);
    ///
    /// let a = root.child(0).unwrap();
    /// let data: Vec<_> = a.children().map(|x| *x.data()).collect();
    /// assert_eq!(data, ['c', 'd', 'e']);
    ///
    /// let b = root.child(1).unwrap();
    /// let data: Vec<_> = b.children().map(|x| *x.data()).collect();
    /// assert_eq!(data, ['f', 'g']);
    /// ```
    pub fn child_mut(self, child_index: usize) -> Option<NodeMut<'a, V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }

    // helpers

    pub(crate) fn new(col: &'a mut SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Self { col, node_ptr }
    }

    fn node_mut(&mut self) -> &mut N<V> {
        unsafe { &mut *self.node_ptr().ptr_mut() }
    }
}
