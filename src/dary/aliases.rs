use super::Dary;
use crate::{memory::Auto, pinned_storage::SplitRecursive, Node, Tree};

/// A binary tree where each of the nodes might have at most 2 children.
pub type Binary<T> = Dary<2, T>;

/// A d-ary tree where each of the nodes might have at most `D` children.
///
/// # Examples
///
/// ```
/// use orx_tree::*;
///
/// // # A. BUILDING A TREE
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
/// let mut tree = DaryTree::<4, _>::new(1i32); // each node can have at most 4 children
///
/// let mut root = tree.root_mut();
/// let [id2, id3] = root.push_children([2, 3]);
/// let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
/// let id8 = tree.node_mut(&id4).push_child(8);
/// let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
/// let id9 = tree.node_mut(&id6).push_child(9);
/// tree.node_mut(&id7).push_children([10, 11]);
///
/// // traversals
///
/// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
/// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
///
/// let dfs: Vec<_> = tree.node(&id3).walk::<Dfs>().copied().collect();
/// assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);
///
/// let post_order: Vec<_> = tree.node(&id3).walk::<PostOrder>().copied().collect();
/// assert_eq!(post_order, [9, 6, 10, 11, 7, 3]);
///
/// let leaves: Vec<_> = tree.root().leaves::<Dfs>().copied().collect();
/// assert_eq!(leaves, [8, 5, 9, 10, 11]);
///
/// let node3 = tree.node(&id3);
/// let paths: Vec<Vec<_>> = node3.paths::<Bfs>().map(|p| p.copied().collect()).collect();
/// assert_eq!(paths, [[9, 6, 3], [10, 7, 3], [11, 7, 3]]);
/// ```
///
/// # Type Aliases and Generic Parameters
///
/// Below is the list of pairs of tree & node type aliases from the simplest to the most complex.
///
/// Note that the generic parameter `P` can almost always be omitted since the default storage is almost always preferable.
///
/// Generic parameter `M` can also be omitted in most cases to use the default auto reclaim policy.
/// Therefore, we can use the simplest type signatures.
/// However, in certain situations it is preferable to use the *never* reclaim policy which guarantees that the node indices
/// will always remain valid.
///
/// Please see the relevant documentations of [`NodeIdx`] and [`MemoryPolicy`].
///
/// [`NodeIdx`]: crate::NodeIdx
/// [`MemoryPolicy`]: crate::MemoryPolicy
///
/// *Type aliases with default pinned vector storage and default memory reclaim policy:*
///
/// ```ignore
/// DaryTree<D, T>     ==> Tree<Dary<D, T>>
/// DaryNode<'a, D, T> ==> Node<'a, Dary<D, T>>
/// ```
///
/// *Type aliases with default pinned vector storage (recommended):*
///
/// ```ignore
/// DaryTree<D, T, M>     ==> Tree<Dary<D, T>, M>
/// DaryNode<'a, D, T, M> ==> Node<'a, Dary<D, T>, M>
/// ```
///
/// *The most general type aliases:*
///
/// ```ignore
/// DaryTree<D, T, M, P>     ==> Tree<Dary<D, T>, M, P>
/// DaryNode<'a, D, T, M, P> ==> Node<'a, Dary<D, T>, M, P>
/// ```
pub type DaryTree<const D: usize, T, M = Auto, P = SplitRecursive> = Tree<Dary<D, T>, M, P>;

/// A binary tree where each node might have 0, 1 or 2 children.
///
/// # Examples
///
/// ```
/// use orx_tree::*;
///
/// // # A. BUILDING A TREE
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
/// let mut tree = BinaryTree::new(1i32); // each node can have at most 2 children
///
/// let mut root = tree.root_mut();
/// let [id2, id3] = root.push_children([2, 3]);
/// let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
/// let id8 = tree.node_mut(&id4).push_child(8);
/// let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
/// let id9 = tree.node_mut(&id6).push_child(9);
/// tree.node_mut(&id7).push_children([10, 11]);
///
/// // traversals
///
/// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
/// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
///
/// let dfs: Vec<_> = tree.node(&id3).walk::<Dfs>().copied().collect();
/// assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);
///
/// let post_order: Vec<_> = tree.node(&id3).walk::<PostOrder>().copied().collect();
/// assert_eq!(post_order, [9, 6, 10, 11, 7, 3]);
///
/// let leaves: Vec<_> = tree.root().leaves::<Dfs>().copied().collect();
/// assert_eq!(leaves, [8, 5, 9, 10, 11]);
///
/// let node3 = tree.node(&id3);
/// let paths: Vec<Vec<_>> = node3.paths::<Bfs>().map(|p| p.copied().collect()).collect();
/// assert_eq!(paths, [[9, 6, 3], [10, 7, 3], [11, 7, 3]]);
/// ```
///
/// # Type Aliases and Generic Parameters
///
/// Below is the list of pairs of tree & node type aliases from the simplest to the most complex.
///
/// Note that the generic parameter `P` can almost always be omitted since the default storage is almost always preferable.
///
/// Generic parameter `M` can also be omitted in most cases to use the default auto reclaim policy.
/// Therefore, we can use the simplest type signatures.
/// However, in certain situations it is preferable to use the *never* reclaim policy which guarantees that the node indices
/// will always remain valid.
///
/// Please see the relevant documentations of [`NodeIdx`] and [`MemoryPolicy`].
///
/// [`NodeIdx`]: crate::NodeIdx
/// [`MemoryPolicy`]: crate::MemoryPolicy
///
/// *Type aliases with default pinned vector storage and default memory reclaim policy:*
///
/// ```ignore
/// BinaryTree<T>     ==> Tree<Dary<2, T>>
/// BinaryNode<'a, T> ==> Node<'a, Dary<2, T>>
/// ```
///
/// *Type aliases with default pinned vector storage (recommended):*
///
/// ```ignore
/// BinaryTree<T, M>     ==> Tree<Dary<2, T>, M>
/// BinaryNode<'a, T, M> ==> Node<'a, Dary<2, T>, M>
/// ```
///
/// *The most general type aliases:*
///
/// ```ignore
/// BinaryTree<T, M, P>     ==> Tree<Dary<2, T>, M, P>
/// BinaryNode<'a, T, M, P> ==> Node<'a, Dary<2, T>, M, P>
/// ```
pub type BinaryTree<T, M = Auto, P = SplitRecursive> = Tree<Binary<T>, M, P>;

// nodes

/// Node of a [`DaryTree`].
pub type DaryNode<'a, const D: usize, T, M = Auto, P = SplitRecursive> = Node<'a, Dary<D, T>, M, P>;

/// Node of a [`BinaryTree`].
pub type BinaryNode<'a, T, M = Auto, P = SplitRecursive> = Node<'a, Dary<2, T>, M, P>;
