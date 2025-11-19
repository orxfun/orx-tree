use crate::{
    TreeVariant,
    subtrees_within::{ClonedSubTreeWithin, CopiedSubTreeWithin, MovedSubTreeWithin},
};
use core::fmt::Debug;

pub(crate) const INVALID_IDX_ERROR: &str = "\n
NodeIdx is not valid for the given tree. Please see the notes and examples of NodeIdx and MemoryPolicy:
* https://docs.rs/orx-tree/latest/orx_tree/struct.NodeIdx.html
* https://docs.rs/orx-tree/latest/orx_tree/trait.MemoryPolicy.html

Specifically, see the example in the following chapter to prevent invalid indices:
* https://docs.rs/orx-tree/latest/orx_tree/trait.MemoryPolicy.html#lazy-memory-claim-preventing-invalid-indices
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
/// We can add child nodes by [`push_child`], [`push_children`] and [`extend_children`] methods.
/// These methods return the indices of the created nodes.
///
/// Similarly, horizontal growth methods [`push_sibling`], [`push_siblings`] and [`extend_siblings`]
/// also return the indices of new nodes.
///
/// [`push_child`]: crate::NodeMut::push_child
/// [`push_children`]: crate::NodeMut::push_children
/// [`extend_children`]: crate::NodeMut::extend_children
/// [`push_sibling`]: crate::NodeMut::push_sibling
/// [`push_siblings`]: crate::NodeMut::push_siblings
/// [`extend_siblings`]: crate::NodeMut::extend_siblings
///
/// **adding a single child: push_child**
///
/// ```
/// use orx_tree::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
///
/// let mut tree = DynTree::new(1);
///
/// let mut root = tree.root_mut();
///
/// let id2 = root.push_child(2);
/// let id3 = root.push_child(3);
///
/// // use id3 to directly access node 3
/// let n3 = tree.node(&id3);
/// assert_eq!(n3.data(), &3);
/// ```
///
/// **adding a constant number of children: push_children**
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
/// let mut tree = DynTree::new(1);
///
/// let mut root = tree.root_mut();
///
/// let [id2, id3] = root.push_children([2, 3]);
///
/// let [id4, id5] = root.push_children([4, 5]);
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
/// let mut tree = DynTree::new(1);
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
/// let mut tree = DynTree::new(1);
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
/// let n2 = root.child(0);
/// let n5 = n2.child(1);
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
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲
/// // 4   5
///
/// let mut tree = DynTree::new(1);
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
/// // or we can use the shorthand:
/// let indices: Vec<_> = root.indices::<Bfs>().collect();
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

impl<V: TreeVariant> NodeIdx<V> {
    /// Creates a subtree view including this node as the root and all of its descendants with their orientation relative
    /// to this node.
    ///
    /// Consuming the created subtree in methods such as [`push_child_tree_within`] or [`push_sibling_tree_within`] will remove the
    /// subtree from its current position to the target position of the same tree.
    ///
    /// Otherwise, it has no impact on the tree.
    ///
    /// [`push_child_tree_within`]: crate::NodeMut::push_child_tree_within
    /// [`push_sibling_tree_within`]: crate::NodeMut::push_sibling_tree_within
    pub fn into_subtree_within(&self) -> MovedSubTreeWithin<V> {
        MovedSubTreeWithin::new(*self)
    }

    /// Creates a subtree view including this node as the root and all of its descendants with their orientation relative
    /// to this node.
    ///
    /// Consuming the created subtree in methods such as [`push_child_tree_within`] or [`push_sibling_tree_within`] will create
    /// the same subtree structure in the target position with cloned values.
    /// This subtree remains unchanged.
    ///
    /// [`push_child_tree_within`]: crate::NodeMut::push_child_tree_within
    /// [`push_sibling_tree_within`]: crate::NodeMut::push_sibling_tree_within
    pub fn as_cloned_subtree_within(&self) -> ClonedSubTreeWithin<V>
    where
        V::Item: Clone,
    {
        ClonedSubTreeWithin::new(*self)
    }

    /// Creates a subtree view including this node as the root and all of its descendants with their orientation relative
    /// to this node.
    ///
    /// Consuming the created subtree in methods such as [`push_child_tree_within`] or [`push_sibling_tree_within`] will create
    /// the same subtree structure in the target position with copied values.
    /// This subtree remains unchanged.
    ///
    /// [`push_child_tree_within`]: crate::NodeMut::push_child_tree_within
    /// [`push_sibling_tree_within`]: crate::NodeMut::push_sibling_tree_within
    pub fn as_copied_subtree_within(&self) -> CopiedSubTreeWithin<V>
    where
        V::Item: Copy,
    {
        CopiedSubTreeWithin::new(*self)
    }
}

impl<V: TreeVariant> core::hash::Hash for NodeIdx<V> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
// Only the pointer is copied, so "V" does not need to be copy itself.
impl<V: TreeVariant> Copy for NodeIdx<V> {}

impl<V: TreeVariant> Clone for NodeIdx<V> {
    fn clone(&self) -> Self {
        *self
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
