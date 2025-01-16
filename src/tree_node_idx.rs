use core::fmt::Debug;

use crate::TreeVariant;

pub(crate) const INVALID_IDX_ERROR: &str = "\n
NodeIdx is not valid for the given tree.
Please see the notes and examples of NodeIdx and MemoryPolicy:
* https://docs.rs/orx-tree/latest/orx_tree/struct.NodeIdx.html
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
/// We can add nodes to the tree by [`push_child`] and [`push_children`] methods.
/// These methods only create the nodes.
/// If we want to receive the indices of the created nodes at the same time,
/// we can use the [`grow`] and [`extend_children`] methods instead.
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
/// let mut root = tree.root_mut();
///
/// root.push_child(2); // no idx is returned
///
/// let [id3] = root.push_children([3]); // idx is received
///
/// // use id3 to directly access node 3
/// let n3 = tree.node(&id3);
/// assert_eq!(n3.data(), &3);
/// ```
///
/// **adding a constant number of children: push_children vs grow**
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
/// let mut root = tree.root_mut();
///
/// root.push_children([2, 3]); // no indices are returned
///
/// let [id4, id5] = root.push_children([4, 5]); // indices are received
/// ```
///
/// **adding a variable number of children: extend_children**
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
/// let mut root = tree.root_mut();
///
/// // indices are collected into a vec
/// let indices: Vec<_> = root.extend_children(2..6).collect();
///
/// let id5 = &indices[3];
/// let n5 = tree.node(&id5);
/// assert_eq!(n5.data(), &5);
/// ```
///
/// [`push_child`]: crate::NodeMut::push_child
/// [`push_children`]: crate::NodeMut::push_children
/// [`grow`]: crate::NodeMut::grow
/// [`extend_children`]: crate::NodeMut::extend_children
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
/// let mut root = tree.root_mut();
///
/// let [id2, _] = root.push_children([2, 3]);
///
/// let mut n2 = tree.node_mut(&id2);
/// n2.push_children([4, 5]);
///
/// // task: access node 5 and get its index
/// let root = tree.root();
/// let n2 = root.child(0).unwrap();
/// let n5 = n2.child(1).unwrap();
/// let id5 = n5.idx();
///
/// // now we can use idx5 to directly access node 5
/// let n5 = tree.node(&id5);
/// assert_eq!(n5.data(), &5);
/// assert_eq!(n5.parent(), Some(tree.node(&id2)));
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
/// let mut root = tree.root_mut();
///
/// let [id2, _] = root.push_children([2, 3]);
///
/// let mut n2 = tree.node_mut(&id2);
/// n2.push_children([4, 5]);
///
/// // task: collect all indices in breadth first order
/// let mut bfs = Bfs::default().over_nodes();
/// let root = tree.root();
/// let indices: Vec<_> = root.walk_with(&mut bfs).map(|x| x.idx()).collect();
///
/// // now we can use indices to directly access nodes
/// let id5 = &indices[4];
/// let n5 = tree.node(&id5);
/// assert_eq!(n5.data(), &5);
/// assert_eq!(n5.parent(), Some(tree.node(&id2)));
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
pub struct NodeIdx<V: TreeVariant>(pub(crate) orx_selfref_col::NodeIdx<V>);

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

impl<V: TreeVariant> Debug for NodeIdx<V> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
