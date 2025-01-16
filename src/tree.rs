use crate::{
    helpers::Col,
    iter::AncestorsIterPtr,
    memory::{Auto, MemoryPolicy},
    pinned_storage::{PinnedStorage, SplitRecursive},
    tree_node_idx::INVALID_IDX_ERROR,
    tree_variant::RefsChildren,
    Node, NodeIdx, NodeMut, NodeSwapError, TreeVariant,
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
    /// let [_, idx] = root.push_children([4, 2]);
    ///
    /// assert_eq!(tree.len(), 3);
    ///
    /// let mut node = tree.node_mut(&idx);
    /// node.push_child(7);
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

        NodeIdx(root_idx)
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
    /// root.push_child(4);
    /// let [idx] = root.push_children([2]);
    ///
    /// let mut node = tree.node_mut(&idx);
    /// node.push_child(7);
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
    /// let [b, c] = root.push_children(['b', 'c']);
    /// tree.node_mut(&b).push_child('d');
    /// tree.node_mut(&c).push_children(['e', 'f']);
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
    /// root.push_child('b');
    /// let idx = root.push_child('c');
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
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let n2 = tree.node(&id2);
    /// assert_eq!(n2.data(), &2);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// n3.push_children([4, 5]);
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
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let n2 = tree.node(&id2);
    /// assert_eq!(n2.data(), &2);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// n3.push_children([4, 5]);
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

    // move nodes

    /// ***O(1)*** Swaps the nodes together with their subtrees rooted at the given `first_idx` and `second_idx`
    /// in constant time (*).
    /// Returns the error if the swap operation is invalid.
    ///
    /// In order to have a valid swap operation, the two subtrees must be **independent** of each other without
    /// any shared node. Necessary and sufficient condition is then as follows:
    ///
    /// * node with the `first_idx` is not an ancestor of the node with the `second_idx`,
    /// * and vice versa.
    ///
    /// Swap operation will succeed and return Ok if both indices are valid and the above condition holds.
    /// It will the corresponding [`NodeSwapError`] otherwise.
    ///
    /// # See also
    ///
    /// (*) Validation of the independence of the subtrees is performed in ***O(D)*** time where D is the maximum
    /// depth of the tree. When we are certain that the subtrees do not intersect, we can use the unsafe variant
    /// [`swap_nodes_unchecked`] to bypass the validation and avoid the O(D) time.
    ///
    /// In order to only swap data of two nodes, rather than the subtrees, please see [`swap_data_with`] method.
    ///
    /// [`swap_nodes_unchecked`]: crate::Tree::swap_nodes_unchecked
    /// [`swap_data_with`]: crate::NodeMut::swap_data_with
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
    /// //  ╱ ╲   ╱ ╲
    /// // 4   5 6   7
    /// // |     |  ╱ ╲
    /// // 8     9 10  11
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let id1 = root.idx();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// let [id10, _] = tree.node_mut(&id7).push_children([10, 11]);
    ///
    /// // cannot swap n3 & n10
    ///
    /// assert_eq!(
    ///     tree.swap_nodes(&id3, &id10),
    ///     Err(NodeSwapError::FirstNodeIsAncestorOfSecond)
    /// );
    ///
    /// // cannot swap n4 & n1 (root)
    ///
    /// assert_eq!(
    ///     tree.swap_nodes(&id4, &id1),
    ///     Err(NodeSwapError::SecondNodeIsAncestorOfFirst)
    /// );
    ///
    /// // we can swap n2 & n5
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   7     3
    /// //  ╱ ╲   ╱ ╲
    /// // 10 11 6   2
    /// //       |  ╱ ╲
    /// //       9 4   5
    /// //         |
    /// //         8
    ///
    /// let result = tree.swap_nodes(&id2, &id7);
    /// assert_eq!(result, Ok(()));
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 7, 3, 10, 11, 6, 2, 9, 4, 5, 8]);
    /// ```
    pub fn swap_nodes(
        &mut self,
        first_idx: &NodeIdx<V>,
        second_idx: &NodeIdx<V>,
    ) -> Result<(), NodeSwapError> {
        let ptr_root = match self.root_ptr() {
            Some(x) => x,
            None => return Err(NodeSwapError::NodeIdxError(NodeIdxError::RemovedNode)),
        };
        let ptr_p = self.0.try_get_ptr(&first_idx.0)?;
        let ptr_q = self.0.try_get_ptr(&second_idx.0)?;

        if ptr_p == ptr_q {
            Ok(())
        } else if AncestorsIterPtr::new(ptr_root.clone(), ptr_p.clone()).any(|x| x == ptr_q) {
            Err(NodeSwapError::SecondNodeIsAncestorOfFirst)
        } else if AncestorsIterPtr::new(ptr_root.clone(), ptr_q.clone()).any(|x| x == ptr_p) {
            Err(NodeSwapError::FirstNodeIsAncestorOfSecond)
        } else {
            let p = unsafe { &mut *ptr_p.ptr_mut() };
            let q = unsafe { &mut *ptr_q.ptr_mut() };

            let parent_p = p.prev().get().cloned();
            let parent_q = q.prev().get().cloned();

            match parent_p {
                Some(parent_ptr_p) => {
                    let parent_p = unsafe { &mut *parent_ptr_p.ptr_mut() };
                    parent_p.next_mut().replace_with(&ptr_p, ptr_q.clone());

                    q.prev_mut().set_some(parent_ptr_p);
                }
                None => {
                    q.prev_mut().set_none();
                }
            }

            match parent_q {
                Some(parent_ptr_q) => {
                    let parent_q = unsafe { &mut *parent_ptr_q.ptr_mut() };
                    parent_q.next_mut().replace_with(&ptr_q, ptr_p);

                    p.prev_mut().set_some(parent_ptr_q);
                }
                None => {
                    p.prev_mut().set_none();
                }
            }

            Ok(())
        }
    }

    /// ***O(1)*** Swaps the nodes together with their subtrees rooted at the given `first_idx` and `second_idx`
    /// in constant time.
    ///
    /// In order to have a valid swap operation, the two subtrees must be **independent** of each other without
    /// any shared node. Necessary and sufficient condition is then as follows:
    ///
    /// * node with the `first_idx` is not an ancestor of the node with the `second_idx`,
    /// * and vice versa.
    ///
    /// # Panics
    ///
    /// Panics if either of the node indices is invalid.
    ///
    /// # Safety
    ///
    /// It is safe to use this method only when the swap operation is valid; i.e., abovementioned independence condition
    /// of the subtrees holds.
    ///
    /// See also the [`swap_nodes`] method for a safe variant.
    ///
    /// [`swap_nodes`]: Tree::swap_nodes
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
    /// //  ╱ ╲   ╱ ╲
    /// // 4   5 6   7
    /// // |     |  ╱ ╲
    /// // 8     9 10  11
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// let [_, _] = tree.node_mut(&id7).push_children([10, 11]);
    ///
    /// // we can swap n2 & n5
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   7     3
    /// //  ╱ ╲   ╱ ╲
    /// // 10 11 6   2
    /// //       |  ╱ ╲
    /// //       9 4   5
    /// //         |
    /// //         8
    ///
    /// unsafe { tree.swap_nodes_unchecked(&id2, &id7) };
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 7, 3, 10, 11, 6, 2, 9, 4, 5, 8]);
    /// ```
    pub unsafe fn swap_nodes_unchecked(&mut self, first_idx: &NodeIdx<V>, second_idx: &NodeIdx<V>) {
        assert!(self.is_node_idx_valid(first_idx), "{}", INVALID_IDX_ERROR);
        assert!(self.is_node_idx_valid(second_idx), "{}", INVALID_IDX_ERROR);

        let ptr_p = first_idx.0.node_ptr();
        let ptr_q = second_idx.0.node_ptr();

        if ptr_p == ptr_q {
            return;
        }

        let p = unsafe { &mut *ptr_p.ptr_mut() };
        let q = unsafe { &mut *ptr_q.ptr_mut() };

        let parent_p = p.prev().get().cloned();
        let parent_q = q.prev().get().cloned();

        match parent_p {
            Some(parent_ptr_p) => {
                let parent_p = unsafe { &mut *parent_ptr_p.ptr_mut() };
                parent_p.next_mut().replace_with(&ptr_p, ptr_q.clone());

                q.prev_mut().set_some(parent_ptr_p);
            }
            None => {
                q.prev_mut().set_none();
            }
        }

        match parent_q {
            Some(parent_ptr_q) => {
                let parent_q = unsafe { &mut *parent_ptr_q.ptr_mut() };
                parent_q.next_mut().replace_with(&ptr_q, ptr_p);

                p.prev_mut().set_some(parent_ptr_q);
            }
            None => {
                p.prev_mut().set_none();
            }
        }

        if p.prev().get().is_none() {
            self.0.ends_mut().set_some(first_idx.0.node_ptr());
        } else if q.prev().get().is_none() {
            self.0.ends_mut().set_some(second_idx.0.node_ptr());
        }
    }

    // helpers

    /// Returns the pointer to the root; None if empty.
    fn root_ptr(&self) -> Option<&NodePtr<V>> {
        self.0.ends().get()
    }
}
