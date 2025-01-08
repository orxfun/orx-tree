use crate::{pinned_storage::PinnedStorage, MemoryPolicy, Node, NodeMut, Tree, TreeVariant};
use orx_selfref_col::{MemoryState, NodeIdxError, NodePtr};

const INVALID_IDX_ERROR: &str = "\n
NodeIdx is not valid for the given tree.
Please see the notes and examples of NodeIdx and MemoryPolicy:\n
* https://docs.rs/orx-tree/latest/orx_tree/struct.NodeIdx.html\n
* https://docs.rs/orx-tree/latest/orx_tree/trait.MemoryPolicy.html\n
\n";

/// An index associated only with the node it is created for.
///
/// * Similar to usize for an array, a `NodeIdx` provides direct constant time access to the
///   node it is created for.
///   Therefore, node indices are crucial for efficiency of certain programs.
/// * Unlike usize for an array, a `NodeIdx` is specific which provides additional safety features.
///   * A node index is specific to only one node that it is created for, it can never return another node.
///   * If we create a node index from one tree and use it on another tree, we get an error ([`OutOfBounds`]).
///   * If we create a node index for a node, then we remove this node from the tree, and then we use
///     the index, we get an error ([`RemovedNode`]).
///   * If we create a node index for a node, then the nodes of the tree are reorganized to reclaim memory,
///     we get an error ([`ReorganizedCollection`]) when we try to use the node index.
///     This error is due to an implicit operation which is undesirable.
///     However, we can conveniently avoid such errors using [`Auto`] and [`Lazy`] memory reclaim policies
///     together. Please see the notes and examples in the [`MemoryPolicy`].
///
/// [`OutOfBounds`]: crate::NodeIdxError::OutOfBounds
/// [`RemovedNode`]: crate::NodeIdxError::RemovedNode
/// [`ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
/// [`Auto`]: crate::Auto
/// [`Lazy`]: crate::Lazy
/// [`MemoryPolicy`]: crate::MemoryPolicy
///
/// # Collecting Node Indices
///
/// There are three ways to get the index of a node.
///
/// ## 1. During Growth
///
/// We can add nodes to the tree by [`push`] and [`extend`] methods.
/// These methods only create the nodes.
/// If we want to receive the indices of the created nodes at the same time,
/// we can use the [`grow`], [`grow_iter`] and [`grow_vec`] methods instead.
///
/// **adding a single child: push vs grow**
///
/// ```
/// use orx_tree::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
///
/// root.push(2); // no idx is returned
///
/// let [id3] = root.grow([3]); // idx is received
///
/// // use id3 to directly access node 3
/// let n3 = id3.node(&tree);
/// assert_eq!(n3.data(), &3);
/// ```
///
/// **adding a constant number of children: extend vs grow**
///
/// ```
/// use orx_tree::*;
///
/// //       1
/// //      ╱|╲
/// //     ╱ | ╲
/// //    ╱ ╱╲  ╲
/// //   2 3  4  5
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
///
/// root.extend([2, 3]); // no indices are returned
///
/// let [id4, id5] = root.grow([4, 5]); // indices are received
/// ```
///
/// **adding a variable number of children: extend vs grow_iter or grow_vec**
///
/// ```
/// use orx_tree::*;
///
/// //       1
/// //      ╱|╲
/// //     ╱ | ╲
/// //    ╱ ╱╲  ╲
/// //   2 3  4  5
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
///
/// root.extend(2..4); // no indices are returned
///
/// let indices = root.grow_vec(4..6); // indices are collected into a vec
///
/// let id5 = &indices[1];
/// let n5 = id5.node(&tree);
/// assert_eq!(n5.data(), &5);
/// ```
///
/// [`push`]: crate::NodeMut::push
/// [`extend`]: crate::NodeMut::extend
/// [`grow`]: crate::NodeMut::grow
/// [`grow_iter`]: crate::NodeMut::grow_iter
/// [`grow_vec`]: crate::NodeMut::grow_vec
///
/// ## 2. From the Node
///
/// A node index can be obtained from the node itself using the [`idx`] method.
/// There are different ways to access the nodes:
/// * we can traverse the tree ourselves using child and parent methods,
/// * or we can traverse the tree [`OverNode`].
///
/// [`idx`]: crate::NodeRef::idx
/// [`OverNode`]: crate::traversal::OverNode
///
/// ```
/// use orx_tree::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲
/// // 4   5
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
///
/// let [id2, _] = root.grow([2, 3]);
///
/// let mut n2 = id2.node_mut(&mut tree);
/// n2.extend([4, 5]);
///
/// // task: access node 5 and get its index
/// let root = tree.root().unwrap();
/// let n2 = root.child(0).unwrap();
/// let n5 = n2.child(1).unwrap();
/// let id5 = n5.idx();
///
/// // now we can use idx5 to directly access node 5
/// let n5 = id5.node(&tree);
/// assert_eq!(n5.data(), &5);
/// assert_eq!(n5.parent(), Some(id2.node(&tree)));
/// ```
///
/// Since we can traverse the node in various ways and access the nodes in various orders,
/// we can also collect the indices in desired order.
///
/// ```
/// use orx_tree::*;
/// use orx_tree::traversal::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲
/// // 4   5
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
///
/// let [id2, _] = root.grow([2, 3]);
///
/// let mut n2 = id2.node_mut(&mut tree);
/// n2.extend([4, 5]);
///
/// // task: collect all indices in breadth first order
/// let root = tree.root().unwrap();
/// let indices: Vec<_> = root.bfs_over::<OverNode>().map(|x| x.idx()).collect();
///
/// // now we can use indices to directly access nodes
/// let id5 = &indices[4];
/// let n5 = id5.node(&tree);
/// assert_eq!(n5.data(), &5);
/// assert_eq!(n5.parent(), Some(id2.node(&tree)));
/// ```
///
/// # Validity of Node Indices
///
/// At the time it is created, the node index:
///
/// * is valid for the tree the node belongs to,
/// * is invalid for any other tree:
///   * `idx.is_valid_for(&other_tree)` => false
///   * `idx.node(&other_tree)` => panics!!!
///   * `idx.get_node(&other_tree)` => None
///   * `idx.try_get_node(&other_tree)` => Err([`OutOfBounds`])
///
/// However, it might later become invalid for the original tree due to two reasons.
///
/// The first reason is explicit.
/// If the node is removed from the tree, directly or due to removal of any of its ancestors,
/// the corresponding index becomes invalid:
/// * `idx.is_valid_for(&correct_tree)` => false
/// * `idx.node(&correct_tree)` => panics!!!
/// * `idx.get_node(&correct_tree)` => None
/// * `idx.try_get_node(&correct_tree)` => Err([`RemovedNode`])
///
/// The second reason is implicit and closely related to [`MemoryPolicy`].
/// If removals from the tree triggers a memory reclaim operation which reorganizes the nodes of
/// the tree, all indices cached prior to the reorganization becomes invalid:
/// * `idx.is_valid_for(&correct_tree)` => false
/// * `idx.node(&correct_tree)` => panics!!!
/// * `idx.get_node(&correct_tree)` => None
/// * `idx.try_get_node(&correct_tree)` => Err([`ReorganizedCollection`])
///
/// The implicit invalidation is not desirable and can be avoided by using memory policies,
/// please see the [`MemoryPolicy`] documentation and examples.
/// In brief:
/// * [`Lazy`] policy never leads to implicit invalidation.
/// * Growth methods never lead to implicit invalidation.
/// * We can only experience implicit invalidation when we are using [`Auto`] (or auto with threshold)
///   memory policy and remove nodes from the tree.
pub struct NodeIdx<V: TreeVariant>(orx_selfref_col::NodeIdx<V>);

impl<V: TreeVariant> Clone for NodeIdx<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: TreeVariant> PartialEq for NodeIdx<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V: TreeVariant> NodeIdx<V> {
    #[inline(always)]
    pub(crate) fn new(state: MemoryState, node_ptr: &NodePtr<V>) -> Self {
        Self(orx_selfref_col::NodeIdx::new(state, node_ptr))
    }

    /// Returns true if this node index is valid for the given `tree`.
    ///
    /// Returns false if either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn is_valid_for<M, P>(&self, tree: &Tree<V, M, P>) -> bool
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        self.0.is_valid_for(&tree.0)
    }

    /// Returns the node that this index is pointing to in constant time.
    ///
    /// # Panics
    ///
    /// Panics if this node index is not valid for the given `tree`; i.e., when either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.0.is_valid_for(&tree.0), "{}", INVALID_IDX_ERROR);
        Node::new(&tree.0, self.0.node_ptr())
    }

    /// Returns the mutable node that this index is pointing to in constant time.
    ///
    /// # Panics
    ///
    /// Panics if this node index is not valid for the given `tree`; i.e., when either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.0.is_valid_for(&tree.0), "{}", INVALID_IDX_ERROR);
        NodeMut::new(&mut tree.0, self.0.node_ptr())
    }

    /// Returns the node that this index is pointing to in constant time if the index is valid.
    ///
    /// Returns None when either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn get_node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Option<Node<'a, V, M, P>>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        self.0
            .is_valid_for(&tree.0)
            .then(|| Node::new(&tree.0, self.0.node_ptr()))
    }

    /// Returns the mutable node that this index is pointing to in constant time if the index is valid.
    ///
    /// Returns None when either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn get_node_mut<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> Option<NodeMut<'a, V, M, P>>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        self.0
            .is_valid_for(&tree.0)
            .then(|| NodeMut::new(&mut tree.0, self.0.node_ptr()))
    }

    /// Returns the node that this index is pointing to in constant time if the index is valid.
    ///
    /// Returns the node index error when either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn try_get_node<'a, M, P>(
        &self,
        tree: &'a Tree<V, M, P>,
    ) -> Result<Node<'a, V, M, P>, NodeIdxError>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        tree.0
            .try_get_ptr(&self.0)
            .map(|ptr| Node::new(&tree.0, ptr))
    }

    /// Returns the mutable node that this index is pointing to in constant time if the index is valid.
    ///
    /// Returns the node index error when either of the following holds:
    /// * the node index is created from a different tree => [`NodeIdxError::OutOfBounds`]
    /// * the node that this index is created for is removed from the tree => [`NodeIdxError::RemovedNode`]
    /// * the tree is using `Auto` memory reclaim policy and nodes are reorganized due to node removals
    ///   => [`NodeIdxError::ReorganizedCollection`]
    #[inline(always)]
    pub fn try_get_node_mut<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> Result<NodeMut<'a, V, M, P>, NodeIdxError>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        tree.0
            .try_get_ptr(&self.0)
            .map(|ptr| NodeMut::new(&mut tree.0, ptr))
    }

    /// Returns the node that this index is pointing to in constant time.
    ///
    /// # Safety
    ///
    /// It omits the index validity assertions that [`node`] method performs; hence it is only safe to use
    /// this method when we are certain that `node_idx.is_valid_for(tree)` would have returned true.
    ///
    /// [`node`]: Self::node
    #[inline(always)]
    pub unsafe fn node_unchecked<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        Node::new(&tree.0, self.0.node_ptr())
    }

    /// Returns the mutable node that this index is pointing to in constant time.
    ///
    /// # Safety
    ///
    /// It omits the index validity assertions that [`node_mut`] method performs; hence it is only safe to use
    /// this method when we are certain that `node_idx.is_valid_for(tree)` would have returned true.
    ///
    /// [`node_mut`]: Self::node_mut
    #[inline(always)]
    pub unsafe fn node_mut_unchecked<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> NodeMut<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        NodeMut::new(&mut tree.0, self.0.node_ptr())
    }
}

#[test]
fn abc() {
    use crate::traversal::*;
    use crate::*;
    use alloc::vec::Vec;

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲
    // 4   5

    let mut tree = DynTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();

    let [id2, _] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    n2.extend([4, 5]);

    // task: collect all indices in breadth first order
    let root = tree.root().unwrap();
    let indices: Vec<_> = root.bfs_over::<OverNode>().map(|x| x.idx()).collect();

    // now we can use indices to directly access nodes
    let id5 = &indices[4];
    let n5 = id5.node(&tree);
    assert_eq!(n5.data(), &5);
    assert_eq!(n5.parent(), Some(id2.node(&tree)));
}
