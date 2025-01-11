use crate::{
    helpers::Col,
    memory::{Auto, MemoryPolicy},
    pinned_storage::{PinnedStorage, SplitRecursive},
    tree_node_idx::INVALID_IDX_ERROR,
    Node, NodeIdx, NodeMut, TreeVariant,
};
use orx_selfref_col::{NodeIdxError, NodePtr, RefsSingle};

/// Core tree structure.
pub struct Tree<V, M = Auto, P = SplitRecursive>(pub(crate) Col<V, M, P>)
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage;

impl<V, M, P> Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    /// Creates an empty tree.
    ///
    /// You may call [`push_root`] to instantiate the empty tree.
    ///
    /// [`push_root`]: Self::push_root
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_tree::*;
    ///
    /// let tree: DynTree<i32> = DynTree::empty();
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.get_root(), None);
    /// ```
    pub fn empty() -> Self
    where
        P::PinnedVec<V>: Default,
    {
        Self(Col::<V, M, P>::new())
    }

    /// Creates a new tree including the root node with the given `root_value`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_tree::*;
    ///
    /// let tree: DynTree<i32> = DynTree::new(42);
    ///
    /// assert_eq!(tree.len(), 1);
    /// assert_eq!(tree.get_root().unwrap().data(), &42);
    /// ```
    pub fn new(root_value: V::Item) -> Self
    where
        P::PinnedVec<V>: Default,
    {
        let mut col = Col::<V, M, P>::new();
        let root_ptr = col.push(root_value);
        let root_mut: &mut RefsSingle<V> = col.ends_mut();
        root_mut.set_some(root_ptr);

        Self(col)
    }

    /// ***O(1)*** Returns the number of nodes in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree: DynTree<i32> = DynTree::new(42);
    /// assert_eq!(tree.len(), 1);
    ///
    /// let mut root = tree.root_mut();
    /// let [_, idx] = root.grow([4, 2]);
    ///
    /// assert_eq!(tree.len(), 3);
    ///
    /// let mut node = tree.node_mut(&idx);
    /// node.push(7);
    ///
    /// assert_eq!(tree.len(), 4);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the tree is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Pushes the root to the empty tree.
    ///
    /// # Panics
    ///
    /// Panics if push_root is called when the tree is not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree: DynTree<i32> = DynTree::empty();
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.get_root(), None);
    ///
    /// tree.push_root(42);
    /// assert!(!tree.is_empty());
    /// assert_eq!(tree.len(), 1);
    /// assert_eq!(tree.get_root().unwrap().data(), &42);
    /// ```
    pub fn push_root(&mut self, root_value: V::Item) -> NodeIdx<V> {
        assert!(
            self.is_empty(),
            "Cannot push root to the tree which already has a root."
        );

        let root_idx = self.0.push_get_idx(root_value);
        let root_mut: &mut RefsSingle<V> = self.0.ends_mut();
        root_mut.set_some(root_idx.node_ptr());

        NodeIdx::from_idx(root_idx)
    }

    /// Removes all the nodes including the root of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree: BinaryTree<i32> = BinaryTree::new(42);
    ///
    /// let mut root = tree.root_mut();
    /// root.push(4);
    /// let [idx] = root.grow([2]);
    ///
    /// let mut node = tree.node_mut(&idx);
    /// node.push(7);
    ///
    /// assert_eq!(tree.len(), 4);
    /// assert_eq!(tree.get_root().unwrap().data(), &42);
    ///
    /// tree.clear();
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.get_root(), None);
    /// ```
    pub fn clear(&mut self) {
        self.0.clear();
        self.0.ends_mut().set_none();
    }

    // get root

    /// Returns the root node of the tree.
    ///
    /// # Panics
    ///
    /// Panics if the tree is empty and has no root.
    ///
    /// When not certain, you may use [`is_empty`] or [`get_root`] methods to have a safe access.
    ///
    /// [`is_empty`]: Self::is_empty
    /// [`get_root`]: Self::get_root
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // initiate a rooted tree
    /// let mut tree = DynTree::<_>::new('a');
    /// assert_eq!(tree.root().data(), &'a');
    ///
    /// tree.clear();
    /// // assert_eq!(tree.get_root().data(), 'x'); // panics!
    ///
    /// // initiate an empty tree
    /// let mut tree = BinaryTree::<_>::empty();
    /// // assert_eq!(tree.get_root().data(), 'x'); // panics!
    ///
    /// tree.push_root('a');
    /// assert_eq!(tree.root().data(), &'a');
    /// ```
    pub fn root(&self) -> Node<V, M, P> {
        self.root_ptr()
            .cloned()
            .map(|p| Node::new(&self.0, p))
            .expect("Tree is empty and has no root. You may use `push_root` to add a root and/or `get_root` to safely access the root if it exists.")
    }

    /// Returns the mutable root node of the tree.
    ///
    /// # Panics
    ///
    /// Panics if the tree is empty and has no root.
    ///
    /// When not certain, you may use [`is_empty`] or [`get_root_mut`] methods to have a safe access.
    ///
    /// [`is_empty`]: Self::is_empty
    /// [`get_root_mut`]: Self::get_root_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // initiate a rooted tree
    /// let mut tree = DynTree::<_>::new('a');
    /// *tree.root_mut().data_mut() = 'x';
    /// assert_eq!(tree.root().data(), &'x');
    ///
    /// tree.clear();
    /// // *tree.root_mut().data_mut() = 'x'; // panics!
    ///
    /// // initiate an empty tree
    /// let mut tree = BinaryTree::<_>::empty();
    /// // *tree.root_mut().data_mut() = 'x'; // panics!
    ///
    /// tree.push_root('a');
    ///
    /// // build the tree from the root
    /// let mut root = tree.root_mut();
    /// assert_eq!(root.data(), &'a');
    ///
    /// let [b, c] = root.grow(['b', 'c']);
    /// tree.node_mut(&b).push('d');
    /// tree.node_mut(&c).extend(['e', 'f']);
    /// ```
    pub fn root_mut(&mut self) -> NodeMut<V, M, P> {
        self.root_ptr()
            .cloned()
            .map(|p| NodeMut::new(&mut self.0, p))
            .expect("Tree is empty and has no root. You may use `push_root` to add a root and/or `get_root` to safely access the root if it exists.")
    }

    /// Returns the root node of the tree; None if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // initiate a rooted tree
    /// let mut tree = DynTree::<_>::new('a');
    /// assert_eq!(tree.get_root().unwrap().data(), &'a');
    ///
    /// tree.clear();
    /// assert_eq!(tree.get_root(), None);
    ///
    /// // initiate an empty tree
    /// let mut tree = BinaryTree::<_>::empty();
    /// assert_eq!(tree.get_root(), None);
    ///
    /// tree.push_root('a');
    /// assert_eq!(tree.get_root().unwrap().data(), &'a');
    /// ```
    pub fn get_root(&self) -> Option<Node<V, M, P>> {
        self.root_ptr().cloned().map(|p| Node::new(&self.0, p))
    }

    /// Returns the root as a mutable node of the tree; None if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<_>::new('a');
    ///
    /// let mut root = tree.root_mut();
    ///
    /// assert_eq!(root.data(), &'a');
    /// *root.data_mut() = 'x';
    /// assert_eq!(root.data(), &'x');
    ///
    /// root.push('b');
    /// let idx = root.push('c');
    ///
    /// tree.clear();
    /// assert_eq!(tree.get_root_mut(), None);
    /// ```
    pub fn get_root_mut(&mut self) -> Option<NodeMut<V, M, P>> {
        self.root_ptr()
            .cloned()
            .map(|p| NodeMut::new(&mut self.0, p))
    }

    // get nodes

    /// Returns true if the `node_idx` is valid for this tree.
    ///
    /// Returns false if any of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    #[inline(always)]
    pub fn is_node_idx_valid(&self, node_idx: &NodeIdx<V>) -> bool {
        node_idx.0.is_valid_for(&self.0)
    }

    /// Returns the node with the given `node_idx`.
    ///
    /// # Panics
    ///
    /// Panics if this node index is not valid for the given `tree`; i.e., when either of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// When not certain, you may use [`is_node_idx_valid`] or [`get_node`] methods to have a safe access.
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    ///
    /// [`is_node_idx_valid`]: crate::Tree::is_node_idx_valid
    /// [`get_node`]: Self::get_node
    ///
    /// [`NodeIdxError::OutOfBounds`]: crate::NodeIdxError::OutOfBounds
    /// [`NodeIdxError::RemovedNode`]: crate::NodeIdxError::RemovedNode
    /// [`NodeIdxError::ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //        ╱ ╲
    /// //       4   5
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let n2 = tree.node(&id2);
    /// assert_eq!(n2.data(), &2);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// n3.extend([4, 5]);
    ///
    /// let bfs_values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_values, [1, 2, 3, 4, 5]);
    /// ```
    #[inline(always)]
    pub fn node(&self, node_idx: &NodeIdx<V>) -> Node<V, M, P> {
        assert!(self.is_node_idx_valid(node_idx), "{}", INVALID_IDX_ERROR);
        Node::new(&self.0, node_idx.0.node_ptr())
    }

    /// Returns the mutable node with the given `node_idx`.
    ///
    /// # Panics
    ///
    /// Panics if this node index is not valid for the given `tree`; i.e., when either of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// When not certain, you may use [`is_node_idx_valid`] or [`get_node_mut`] methods to have a safe access.
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    ///
    /// [`is_node_idx_valid`]: crate::Tree::is_node_idx_valid
    /// [`get_node_mut`]: Self::get_node_mut
    ///
    /// [`NodeIdxError::OutOfBounds`]: crate::NodeIdxError::OutOfBounds
    /// [`NodeIdxError::RemovedNode`]: crate::NodeIdxError::RemovedNode
    /// [`NodeIdxError::ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //        ╱ ╲
    /// //       4   5
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let n2 = tree.node(&id2);
    /// assert_eq!(n2.data(), &2);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// n3.extend([4, 5]);
    ///
    /// let bfs_values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_values, [1, 2, 3, 4, 5]);
    /// ```
    #[inline(always)]
    pub fn node_mut(&mut self, node_idx: &NodeIdx<V>) -> NodeMut<V, M, P> {
        assert!(self.is_node_idx_valid(node_idx), "{}", INVALID_IDX_ERROR);
        NodeMut::new(&mut self.0, node_idx.0.node_ptr())
    }

    /// Returns the node with the given `node_idx`; returns None if the node index is invalid.
    ///
    /// The node index is invalid if any of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// You may use [`try_node`] method to get the underlying reason when the index is invalid.
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    ///
    /// [`try_node`]: Self::try_node
    /// [`NodeIdxError::OutOfBounds`]: crate::NodeIdxError::OutOfBounds
    /// [`NodeIdxError::RemovedNode`]: crate::NodeIdxError::RemovedNode
    /// [`NodeIdxError::ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
    #[inline(always)]
    pub fn get_node(&self, node_idx: &NodeIdx<V>) -> Option<Node<V, M, P>> {
        self.is_node_idx_valid(node_idx)
            .then(|| Node::new(&self.0, node_idx.0.node_ptr()))
    }

    /// Returns the mutable node with the given `node_idx`; returns None if the node index is invalid.
    ///
    /// The node index is invalid if any of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// You may use [`try_node_mut`] method to get the underlying reason when the index is invalid.
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    ///
    /// [`try_node_mut`]: Self::try_node_mut
    /// [`NodeIdxError::OutOfBounds`]: crate::NodeIdxError::OutOfBounds
    /// [`NodeIdxError::RemovedNode`]: crate::NodeIdxError::RemovedNode
    /// [`NodeIdxError::ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
    #[inline(always)]
    pub fn get_node_mut(&mut self, node_idx: &NodeIdx<V>) -> Option<NodeMut<V, M, P>> {
        self.is_node_idx_valid(node_idx)
            .then(|| NodeMut::new(&mut self.0, node_idx.0.node_ptr()))
    }

    /// Returns the node with the given `node_idx`; returns the corresponding error if the node index is invalid.
    ///
    /// The node index is invalid if any of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    ///
    /// [`try_node`]: Self::try_node
    /// [`NodeIdxError::OutOfBounds`]: crate::NodeIdxError::OutOfBounds
    /// [`NodeIdxError::RemovedNode`]: crate::NodeIdxError::RemovedNode
    /// [`NodeIdxError::ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
    #[inline(always)]
    pub fn try_node(&self, node_idx: &NodeIdx<V>) -> Result<Node<V, M, P>, NodeIdxError> {
        self.0
            .try_get_ptr(&node_idx.0)
            .map(|ptr| Node::new(&self.0, ptr))
    }

    /// Returns the node with the given `node_idx`; returns the corresponding error if the node index is invalid.
    ///
    /// The node index is invalid if any of the following holds:
    ///
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    ///
    /// Please see [`NodeIdx`] documentation for details on the validity of node indices.
    ///
    /// [`try_node`]: Self::try_node
    /// [`NodeIdxError::OutOfBounds`]: crate::NodeIdxError::OutOfBounds
    /// [`NodeIdxError::RemovedNode`]: crate::NodeIdxError::RemovedNode
    /// [`NodeIdxError::ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
    #[inline(always)]
    pub fn try_node_mut(
        &mut self,
        node_idx: &NodeIdx<V>,
    ) -> Result<NodeMut<V, M, P>, NodeIdxError> {
        self.0
            .try_get_ptr(&node_idx.0)
            .map(|ptr| NodeMut::new(&mut self.0, ptr))
    }

    /// Returns the node with the given `node_idx`.
    ///
    /// # Safety
    ///
    /// It omits the index validity assertions that [`node`] method performs; hence it is only safe to use
    /// this method when we are certain that '`is_node_idx_valid`' would have returned true.
    ///
    /// [`node`]: Self::node
    /// [`is_node_idx_valid`]: Self::is_node_idx_valid
    #[inline(always)]
    pub unsafe fn node_unchecked(&self, node_idx: &NodeIdx<V>) -> Node<V, M, P> {
        Node::new(&self.0, node_idx.0.node_ptr())
    }

    /// Returns the mutable node with the given `node_idx`.
    ///
    /// # Safety
    ///
    /// It omits the index validity assertions that [`node_mut`] method performs; hence it is only safe to use
    /// this method when we are certain that '`is_node_idx_valid`' would have returned true.
    ///
    /// [`node_mut`]: Self::node_mut
    /// [`is_node_idx_valid`]: Self::is_node_idx_valid
    #[inline(always)]
    pub unsafe fn node_mut_unchecked(&mut self, node_idx: &NodeIdx<V>) -> NodeMut<V, M, P> {
        NodeMut::new(&mut self.0, node_idx.0.node_ptr())
    }

    // helpers

    /// Returns the pointer to the root; None if empty.
    fn root_ptr(&self) -> Option<&NodePtr<V>> {
        self.0.ends().get()
    }
}
