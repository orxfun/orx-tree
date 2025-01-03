use super::{BfsIterable, DfsIterable, IterOver, OverData, PostOrderIterable};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// Type to create iterables which are capable of repeatedly creating iterators
/// corresponding to different kinds of traversals starting from different tree nodes
/// without allocating.
///
/// # Traversals
///
/// Currently, it is capable of creating iterables for three kinds of traversals.
///
/// ## dfs (depth first search)
///
/// * dfs iteration internally uses a stack (alloc::vec::Vec).
/// * A dfs iterator can be created from a tree node using [`dfs`], [`dfs_mut`],
///   [`dfs_over`] or [`dfs_mut_over`]. Each time these methods are called a
///   stack will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a dfs iterable using [`TreeIter::dfs`] or
///   [`TreeIter::dfs_over`] methods. These iterables allocate the stack only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// ## bfs (breadth first search)
///
/// * bfs iteration internally uses a queue (alloc::collections::VecDeque).
/// * A bfs iterator can be created from a tree node using [`bfs`], [`bfs_mut`],
///   [`bfs_over`] or [`bfs_mut_over`]. Each time these methods are called a
///   queue will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a bfs iterable using [`TreeIter::bfs`] or
///   [`TreeIter::bfs_over`] methods. These iterables allocate the queue only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// ## post-order
///
/// * post order iteration internally uses a vector (alloc::vec::Vec) of length **D**
///   where D is the maximum depth of nodes visited throughout the traversal.
/// * A post order iterator can be created from a tree node using [`post_order`], [`post_order_mut`],
///   [`post_order_over`] or [`post_order_mut_over`]. Each time these methods are called a
///   vector will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a post order iterable using [`TreeIter::post_order`] or
///   [`TreeIter::post_order_over`] methods. These iterables allocate the vector only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// [`dfs`]: crate::NodeRef::dfs
/// [`dfs_mut`]: crate::NodeMut::dfs_mut
/// [`dfs_over`]: crate::NodeRef::dfs_over
/// [`dfs_mut_over`]: crate::NodeMut::dfs_mut_over
/// [`bfs`]: crate::NodeRef::bfs
/// [`bfs_mut`]: crate::NodeMut::bfs_mut
/// [`bfs_over`]: crate::NodeRef::bfs_over
/// [`bfs_mut_over`]: crate::NodeMut::bfs_mut_over
/// [`post_order`]: crate::NodeRef::post_order
/// [`post_order_mut`]: crate::NodeMut::post_order_mut
/// [`post_order_over`]: crate::NodeRef::post_order_over
/// [`post_order_mut_over`]: crate::NodeMut::post_order_mut_over
pub struct Traversal;

impl Traversal {
    /// Creates a depth-first-search iterable internally using a stack (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    /// See also [`dfs_over`] for other variants.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`dfs_over`]: Self::dfs_over
    /// [`OverData`]: crate::iter::OverData
    ///
    /// # Examples
    ///
    /// The following example demonstrates how the iterable created from [`Traversal`] can be used
    /// to repeatedly iterate over trees without requiring new allocation.
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
    /// let mut root = tree.root_mut().unwrap();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// id4.node_mut(&mut tree).push(8);
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// id6.node_mut(&mut tree).push(9);
    /// id7.node_mut(&mut tree).extend([10, 11]);
    ///
    /// // create the iterable for dfs traversal
    /// // that creates the internal stack once
    ///
    /// let mut dfs = Traversal::dfs();
    ///
    /// // repeatedly create iterators from it, without allocation
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = dfs.iter(&root).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// let mut n7 = id7.node_mut(&mut tree);
    /// for (i, value) in dfs.iter_mut(&mut n7).enumerate() {
    ///     *value += (i * 100) as i32;
    /// }
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = dfs.iter(&n3).copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 110, 211]);
    ///
    /// let n7 = id7.node(&tree);
    /// let values: Vec<_> = dfs.iter(&n7).copied().collect();
    /// assert_eq!(values, [7, 110, 211]);
    /// ```
    pub fn dfs<V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
    ) -> DfsIterable<OverData, V, M, P> {
        Default::default()
    }

    /// Creates a depth-first-search iterable internally using a stack (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse over values defined by the generic parameter [`O`]:
    ///
    /// * [`OverData`] yields data of nodes
    /// * [`OverDepthData`] yields (depth, data) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    pub fn dfs_over<O: IterOver, V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
    ) -> DfsIterable<O, V, M, P> {
        Default::default()
    }

    /// Creates a breadth-first-search iterable internally using a queue (alloc::collections::VecDeque).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    /// See also [`bfs_over`] for other variants.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`bfs_over`]: Self::bfs_over
    /// [`OverData`]: crate::iter::OverData
    ///
    /// # Examples
    ///
    /// The following example demonstrates how the iterable created from [`Traversal`] can be used
    /// to repeatedly iterate over trees without requiring new allocation.
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
    /// let mut root = tree.root_mut().unwrap();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// id4.node_mut(&mut tree).push(8);
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// id6.node_mut(&mut tree).push(9);
    /// id7.node_mut(&mut tree).extend([10, 11]);
    ///
    /// // create the iterable for bfs traversal
    /// // that creates the internal queue once
    ///
    /// let mut bfs = Traversal::bfs();
    ///
    /// // repeatedly create iterators from it, without allocation
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = bfs.iter(&root).copied().collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// let mut n7 = id7.node_mut(&mut tree);
    /// for (i, value) in bfs.iter_mut(&mut n7).enumerate() {
    ///     *value += (i * 100) as i32;
    /// }
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = bfs.iter(&n3).copied().collect();
    /// assert_eq!(values, [3, 6, 7, 9, 110, 211]);
    ///
    /// let n7 = id7.node(&tree);
    /// let values: Vec<_> = bfs.iter(&n7).copied().collect();
    /// assert_eq!(values, [7, 110, 211]);
    /// ```
    pub fn bfs<V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
    ) -> BfsIterable<OverData, V, M, P> {
        Default::default()
    }

    /// Creates a breadth-first-search iterable internally using a queue (alloc::collections::VecDeque).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse over values defined by the generic parameter [`O`]:
    ///
    /// * [`OverData`] yields data of nodes
    /// * [`OverDepthData`] yields (depth, data) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    pub fn bfs_over<O: IterOver, V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
    ) -> BfsIterable<O, V, M, P> {
        Default::default()
    }

    /// Creates an iterable for post-order traversal which internally uses a vector of length **D** where D
    /// is the maximum depth of the nodes traversed during iterations (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree. These iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    ///
    /// Further, [`iter_over`] and [`iter_mut_over`] methods can be used to create iterators which yield
    /// different node value for each visited node. The node values are determined by the generic parameter
    /// of the methods ([`IterOver`]).
    ///
    /// * [`OverData`] yields data of nodes
    /// * [`OverDepthData`] yields (depth, data) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`iter`]: crate::iter::PostOrderIterable::iter
    /// [`iter_mut`]: crate::iter::PostOrderIterable::iter_mut
    /// [`iter_over`]: crate::iter::PostOrderIterable::iter_over
    /// [`iter_mut_over`]: crate::iter::PostOrderIterable::iter_mut_over
    /// [`OverData`]: crate::iter::OverData
    /// [`IterOver`]: crate::iter::IterOver
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    pub fn post_order<V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
    ) -> PostOrderIterable<V, M, P> {
        Default::default()
    }
}
