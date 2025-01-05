use super::{BfsIterable, DfsIterable, IterOver, OverData, PostOrderIterable};
use crate::{helpers::N, TreeVariant};
use core::marker::PhantomData;
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
/// * Alternatively, one can create a dfs iterable using [`TraversalDepr::dfs`] or
///   [`TraversalDepr::over`] methods. These iterables allocate the stack only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// ## bfs (breadth first search)
///
/// * bfs iteration internally uses a queue (alloc::collections::VecDeque).
/// * A bfs iterator can be created from a tree node using [`bfs`], [`bfs_mut`],
///   [`bfs_over`] or [`bfs_mut_over`]. Each time these methods are called a
///   queue will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a bfs iterable using [`TraversalDepr::bfs`] or
///   [`TraversalDepr::over`] methods. These iterables allocate the queue only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// ## post-order
///
/// * post order iteration internally uses a vector (alloc::vec::Vec) of length **D**
///   where D is the maximum depth of nodes visited throughout the traversal.
/// * A post order iterator can be created from a tree node using [`post_order`], [`post_order_mut`],
///   [`post_order_over`] or [`post_order_mut_over`]. Each time these methods are called a
///   vector will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a post order iterable using [`TraversalDepr::post_order`] method.
///   This iterable allocates the vector only once and can be used to repeatedly create iterators
///   without new allocations.
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
#[derive(Default)]
pub struct TraversalDepr;

impl TraversalDepr {
    /// Creates a depth-first-search iterable internally using a stack (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    /// See also [`TraversalDepr::over`] for other variants.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`TraversalDepr::over`]: Self::over
    /// [`OverData`]: crate::iter::OverData
    ///
    /// # Examples
    ///
    /// The following example demonstrates how the iterable created from [`TraversalDepr`] can be used
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
    /// let mut dfs = TraversalDepr::dfs();
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

    /// Creates a breadth-first-search iterable internally using a queue (alloc::collections::VecDeque).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    /// See also [`TraversalDepr::over`] for other variants.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`TraversalDepr::over`]: Self::over
    /// [`OverData`]: crate::iter::OverData
    ///
    /// # Examples
    ///
    /// The following example demonstrates how the iterable created from [`TraversalDepr`] can be used
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
    /// let mut bfs = TraversalDepr::bfs();
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
    /// [`iter`]: crate::iter::PostOrderIterable::iter
    /// [`iter_mut`]: crate::iter::PostOrderIterable::iter_mut
    /// [`iter_over`]: crate::iter::PostOrderIterable::iter_over
    /// [`iter_mut_over`]: crate::iter::PostOrderIterable::iter_mut_over
    ///
    /// # Examples
    ///
    /// The following example demonstrates how the iterable created from [`TraversalDepr`] can be used
    /// to repeatedly iterate over trees without requiring new allocation.
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::iter::*;
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
    /// let [id8] = id4.node_mut(&mut tree).grow([8]);
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// id6.node_mut(&mut tree).push(9);
    /// id7.node_mut(&mut tree).extend([10, 11]);
    ///
    /// // create the iterable for post-order traversal
    /// // that creates the internal vector once
    ///
    /// let mut po = TraversalDepr::post_order();
    ///
    /// // repeatedly create iterators from it, without allocation
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = po.iter(&root).copied().collect();
    /// assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
    ///
    /// let mut n7 = id7.node_mut(&mut tree);
    /// for (i, value) in po.iter_mut(&mut n7).enumerate() {
    ///     *value += (i * 100) as i32;
    /// }
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = po.iter(&n3).copied().collect();
    /// assert_eq!(values, [9, 6, 10, 111, 207, 3]);
    ///
    /// let n7 = id7.node(&tree);
    /// let values: Vec<_> = po.iter(&n7).copied().collect();
    /// assert_eq!(values, [10, 111, 207]);
    ///
    /// // we may also create iterators over other elements
    ///
    /// let root = tree.root().unwrap();
    /// let mut iter = po.iter_over::<OverDepthSiblingData>(&root);
    /// assert_eq!(iter.next(), Some((3, 0, &8)));
    /// assert_eq!(iter.next(), Some((2, 0, &4)));
    /// assert_eq!(iter.next(), Some((2, 1, &5)));
    /// assert_eq!(iter.next(), Some((1, 0, &2)));
    /// assert_eq!(iter.next(), Some((3, 0, &9)));
    ///
    /// let mut iter = po.iter_over::<OverDepthNode>(&root);
    ///
    /// let (d, node) = iter.next().unwrap();
    /// assert_eq!(d, 3);
    /// assert_eq!(&node.idx(), &id8);
    /// assert_eq!(node.num_children(), 0);
    ///
    /// let (d, node) = iter.next().unwrap();
    /// assert_eq!(d, 2);
    /// assert_eq!(&node.idx(), &id4);
    /// assert_eq!(node.num_children(), 1);
    /// assert_eq!(&node.child(0).unwrap().idx(), &id8);
    /// ```
    pub fn post_order<V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
    ) -> PostOrderIterable<V, M, P> {
        Default::default()
    }

    /// Returns the type to create iterables with element type determined by the generic [`IterOver`] parameter `O`.
    pub fn over<O: IterOver>() -> TraversalOver<O> {
        Default::default()
    }
}

/// Type to create iterables which are capable of repeatedly creating iterators
/// corresponding to different kinds of traversals starting from different tree nodes
/// without allocating.
///
/// Return type of the iterators to be created is determined by the generic [`IterOver`] parameter `O`.
pub struct TraversalOver<O: IterOver>(PhantomData<O>);

impl<O: IterOver> Default for TraversalOver<O> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<O: IterOver> TraversalOver<O> {
    /// Creates a depth-first-search iterable internally using a stack (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse over values defined by the generic parameter `O`.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::iter::*;
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
    /// // create the iterable for dfs traversal over (depth, sibling, data)
    /// // that creates the internal stack once
    ///
    /// let mut dfs = TraversalDepr::over::<OverDepthSiblingData>().dfs();
    ///
    /// // repeatedly create iterators from it, without allocation
    ///
    /// // iter from root
    ///
    /// let root = tree.root().unwrap();
    /// let average_depth = dfs
    ///     .iter(&root)
    ///     .map(|(depth, _sibling_idx, _val)| depth)
    ///     .sum::<usize>()
    ///     / tree.len();
    /// assert_eq!(average_depth, 2);
    ///
    /// let num_second_siblings = dfs
    ///     .iter(&root)
    ///     .filter(|(_depth, sibling_idx, _val)| *sibling_idx == 1)
    ///     .count();
    /// assert_eq!(num_second_siblings, 4);
    ///
    /// let values: Vec<i32> = dfs
    ///     .iter(&root)
    ///     .map(|(_depth, _sibling_idx, val)| *val)
    ///     .collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// // or iter_mut
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, value) in dfs.iter_mut(&mut root) {
    ///     if depth + sibling_idx == 3 {
    ///         *value += 100;
    ///     }
    /// }
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<i32> = dfs
    ///     .iter(&root)
    ///     .map(|(_depth, _sibling_idx, val)| *val)
    ///     .collect();
    /// assert_eq!(values, [1, 2, 4, 108, 105, 3, 6, 109, 107, 110, 11]);
    ///
    /// // or from any node
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<i32> = dfs
    ///     .iter(&n3)
    ///     .map(|(_depth, _sibling_idx, val)| *val)
    ///     .collect();
    /// assert_eq!(values, [3, 6, 109, 107, 110, 11]);
    /// ```
    pub fn dfs<V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
        self,
    ) -> DfsIterable<O, V, M, P> {
        Default::default()
    }

    /// Creates a breadth-first-search iterable internally using a queue (alloc::collection::VecDeque).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse over values defined by the generic parameter `O`.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::iter::*;
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
    /// // create the iterable for bfs traversal over (depth, sibling, data)
    /// // that creates the internal stack once
    ///
    /// let mut bfs = TraversalDepr::over::<OverDepthSiblingData>().bfs();
    ///
    /// // repeatedly create iterators from it, without allocation
    ///
    /// // iter from root
    ///
    /// let root = tree.root().unwrap();
    /// let average_depth = bfs
    ///     .iter(&root)
    ///     .map(|(depth, _sibling_idx, _val)| depth)
    ///     .sum::<usize>()
    ///     / tree.len();
    /// assert_eq!(average_depth, 2);
    ///
    /// let num_second_siblings = bfs
    ///     .iter(&root)
    ///     .filter(|(_depth, sibling_idx, _val)| *sibling_idx == 1)
    ///     .count();
    /// assert_eq!(num_second_siblings, 4);
    ///
    /// let values: Vec<i32> = bfs
    ///     .iter(&root)
    ///     .map(|(_depth, _sibling_idx, val)| *val)
    ///     .collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// // or iter_mut
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, value) in bfs.iter_mut(&mut root) {
    ///     if depth + sibling_idx == 3 {
    ///         *value += 100;
    ///     }
    /// }
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<i32> = bfs
    ///     .iter(&root)
    ///     .map(|(_depth, _sibling_idx, val)| *val)
    ///     .collect();
    /// assert_eq!(values, [1, 2, 3, 4, 105, 6, 107, 108, 109, 110, 11]);
    ///
    /// // or from any node
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<i32> = bfs
    ///     .iter(&n3)
    ///     .map(|(_depth, _sibling_idx, val)| *val)
    ///     .collect();
    /// assert_eq!(values, [3, 6, 107, 109, 110, 11]);
    /// ```
    pub fn bfs<V: TreeVariant, M: MemoryPolicy<V>, P: PinnedVec<N<V>>>(
        self,
    ) -> BfsIterable<O, V, M, P> {
        Default::default()
    }
}
