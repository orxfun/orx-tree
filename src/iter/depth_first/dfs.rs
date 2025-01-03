use super::DfsIterMut;
use crate::iter::{
    IterMutOver, IterOver, OverData, OverDepthData, OverDepthNode, OverDepthSiblingData,
    OverDepthSiblingNode, OverNode,
};
use crate::{helpers::N, iter::DfsIter, node_ref::NodeRefCore, NodeMut, NodeRef, TreeVariant};
use alloc::vec::Vec;
use orx_selfref_col::MemoryPolicy;
use orx_split_vec::PinnedVec;

/// Factory to create depth-first-search iterables, which in turn can be used to create iterators rooted from different nodes.
///
/// Note that depth-first-search iterators can directly be created:
/// * from [`NodeRef`] ([`Node`] or [`NodeMut`]) using [`dfs`] or [`dfs_over`] methods, or
/// * from [`NodeMut`] using [`dfs_mut`] or [`dfs_mut_over`] methods.
///
/// Note that the depth first traversal requires a stack.
/// Each time an iterator is crated using above-mentioned tree node methods, a new stack (Vec) is allocated and dropped once the iterator is dropped.
///
/// On the other hand, iterables created from `Dfs` allocate the stack on initialization,
/// and keep re-using the same stack regardless of how many iterators are created from it.
/// This allows to minimize memory allocations and optimize performance in programs where we repeatedly traverse the tree.
///
/// [`Node`]: crate::Node
/// [`dfs`]: crate::NodeRef::dfs
/// [`dfs_over`]: crate::NodeRef::dfs_over
/// [`dfs_mut`]: crate::NodeMut::dfs_mut
/// [`dfs_mut_over`]: crate::NodeMut::dfs_mut_over
pub struct Dfs;

impl Dfs {
    /// Creates an iterable which can be used to create depth-first iterators.
    ///
    /// Item or element type of the created iterators depend on the generic `K: IterOver` parameter:
    ///
    /// ```ignore
    /// Dfs::over::<OverData, _>() <===> Dfs::over_data::<_>()   // yields `data`
    /// Dfs::over::<OverNode, _>() <===> Dfs::over_node::<_>()   // yields `Node`
    ///
    /// Dfs::over::<OverDepthData, _>() <===> Dfs::over_depth_data::<_>()   // yields (depth, `data`)
    /// Dfs::over::<OverDepthNode, _>() <===> Dfs::over_depth_node::<_>()   // yields (depth, `Node`)
    ///
    /// Dfs::over::<OverDepthSiblingData, _>() <===> Dfs::over_depth_sibling_data::<_>()   // yields (depth, sibling_idx, `data`)
    /// Dfs::over::<OverDepthSiblingNode, _>() <===> Dfs::over_depth_sibling_node::<_>()   // yields (depth, sibling_idx, `Node`)
    /// ```
    ///
    /// Depth and sibling indices are based on the node that the created iterator is rooted at:
    /// * depth is the of the node starting from the node that the iterator is created from (root) which has a depth of 0.
    /// * sibling index of the node is its position among its siblings (or children of its parent); root's sibling index is always 0.
    pub fn over<K: IterOver, V: TreeVariant>() -> DfsIterable<V, K> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over [`data`] of nodes.
    ///
    /// A depth first search requires a stack to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::dfs`] method.
    /// However, each time the 'dfs' method is called a new stack is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same stack and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `DfsOverData<V>` which allocates the stack once on initialization;
    /// and creates iterators starting from different nodes that re-use the stack.
    ///
    /// `Dfs::over_data::<V>()` is equivalent to `Dfs::over::<V, OverData>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::dfs`]: crate::NodeRef::dfs
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
    /// let mut tree = BinaryTree::<i32>::new(1);
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
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // its `iter` and `iter_mut` calls re-use the same stack
    /// let mut dfs = Dfs::over_data();
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// for x in dfs.iter_mut(&mut root) {
    ///     *x *= 100;
    /// }
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter(&root);
    /// assert_eq!(iter.next(), Some(&100));
    /// assert_eq!(iter.next(), Some(&200));
    /// assert_eq!(iter.next(), Some(&400));
    /// assert_eq!(iter.next(), Some(&800));
    /// assert_eq!(iter.next(), Some(&500)); // ...
    ///
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = dfs.iter(&n3).copied().collect();
    /// assert_eq!(values, [300, 600, 900, 700, 1000, 1100]);
    /// ```
    pub fn over_data<V: TreeVariant>() -> DfsOverData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the nodes ([`Node`]).
    ///
    /// A depth first search requires a stack to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::dfs_over`] method.
    /// However, each time the 'dfs_over' method is called a new stack is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same stack and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `DfsOverNode<V>` which allocates the stack once on initialization;
    /// and creates iterators starting from different nodes that re-use the stack.
    ///
    /// `Dfs::over_node::<V>()` is equivalent to `Dfs::over::<V, OverNode>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::dfs_over`]: crate::NodeRef::dfs_over
    /// [`Node`]: crate::Node
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
    /// let mut tree = BinaryTree::<i32>::new(1);
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
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // its `iter` calls re-use the same stack
    /// let mut dfs = Dfs::over_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter(&root);
    /// let _n1 = iter.next().unwrap();
    /// let n2 = iter.next().unwrap();
    /// let n4 = iter.next().unwrap();
    /// assert_eq!(n4.data(), &4);
    /// assert_eq!(n4.parent(), Some(n2));
    /// assert_eq!(n4.num_children(), 1);
    ///
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = dfs.iter(&n3).map(|x| *x.data()).collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn over_node<V: TreeVariant>() -> DfsOverNode<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the tuple of depths and nodes.
    ///
    /// * Iterator item => (`depth`, [`data`]) tuple where:
    ///   * depth is the of the node starting from the node that the iterator is created from (root) which has a depth of 0.
    ///
    /// A depth first search requires a stack to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::dfs_over`] method.
    /// However, each time the 'dfs_over' method is called a new stack is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same stack and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `DfsOverDepthData<V>` which allocates the stack once on initialization;
    /// and creates iterators starting from different nodes that re-use the stack.
    ///
    /// `Dfs::over_depth_data::<V>()` is equivalent to `Dfs::over::<V, OverDepthData>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::dfs_over`]: crate::NodeRef::dfs_over
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
    /// let mut tree = BinaryTree::<i32>::new(1);
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
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // its `iter` and `iter_mut` calls re-use the same stack
    /// let mut dfs = Dfs::over_depth_data();
    ///
    /// for (depth, x) in dfs.iter_mut(&mut tree.root_mut().unwrap()) {
    ///     *x += depth as i32 * 100;
    /// }
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter(&root);
    /// assert_eq!(iter.next(), Some((0, &1)));
    /// assert_eq!(iter.next(), Some((1, &102)));
    /// assert_eq!(iter.next(), Some((2, &204)));
    /// assert_eq!(iter.next(), Some((3, &308)));
    /// assert_eq!(iter.next(), Some((2, &205)));
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = dfs.iter(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 1, 2, 2]);
    ///
    /// let values: Vec<_> = dfs.iter(&n3).map(|x| *x.1).collect();
    /// assert_eq!(values, [103, 206, 309, 207, 310, 311]);
    /// ```
    pub fn over_depth_data<V: TreeVariant>() -> DfsOverDepthData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the tuple of depths and nodes.
    ///
    /// * Iterator item => (`depth`, [`Node`]) tuple where:
    ///   * depth is the of the node starting from the node that the iterator is created from (root) which has a depth of 0.
    ///
    /// A depth first search requires a stack to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::dfs_over`] method.
    /// However, each time the 'dfs_over' method is called a new stack is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same stack and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `DfsOverDepthNode<V>` which allocates the stack once on initialization;
    /// and creates iterators starting from different nodes that re-use the stack.
    ///
    /// `Dfs::over_depth_node::<V>()` is equivalent to `Dfs::over::<V, OverDepthNode>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::dfs_over`]: crate::NodeRef::dfs_over
    /// [`Node`]: crate::Node
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
    /// let mut tree = BinaryTree::<i32>::new(1);
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
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // its `iter` calls re-use the same stack
    /// let mut dfs = Dfs::over_depth_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter(&root);
    /// let (d, _n1) = iter.next().unwrap();
    /// assert_eq!(d, 0);
    ///
    /// let (d, n2) = iter.next().unwrap();
    /// assert_eq!(d, 1);
    ///
    /// let (d, n4) = iter.next().unwrap();
    /// assert_eq!(d, 2);
    ///
    /// assert_eq!(n4.data(), &4);
    /// assert_eq!(n4.parent(), Some(n2));
    /// assert_eq!(n4.num_children(), 1);
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = dfs.iter(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 1, 2, 2]);
    ///
    /// let values: Vec<_> = dfs.iter(&n3).map(|x| *x.1.data()).collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn over_depth_node<V: TreeVariant>() -> DfsOverDepthNode<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the tuple of depths, sibling indices and nodes.
    ///
    /// * Iterator item => (`depth`, `sibling_idx` [`data`]) tuple where:
    ///   * depth is the of the node starting from the node that the iterator is created from (root) which has a depth of 0.
    ///   * sibling index of the node is its position among its siblings (or children of its parent); root's sibling index is always 0.
    ///
    /// A depth first search requires a stack to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::dfs_over`] method.
    /// However, each time the 'dfs_over' method is called a new stack is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same stack and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `DfsOverDepthData<V>` which allocates the stack once on initialization;
    /// and creates iterators starting from different nodes that re-use the stack.
    ///
    /// `Dfs::over_depth_sibling_data::<V>()` is equivalent to `Dfs::over::<V, OverDepthSiblingData>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::dfs_over`]: crate::NodeRef::dfs_over
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
    /// let mut tree = BinaryTree::<i32>::new(1);
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
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // its `iter` and `iter_mut` calls re-use the same stack
    /// let mut dfs = Dfs::over_depth_sibling_data();
    ///
    /// for (depth, sibling_idx, x) in dfs.iter_mut(&mut tree.root_mut().unwrap()) {
    ///     match sibling_idx {
    ///         0 => *x += depth as i32 * 100,
    ///         _ => *x = -(*x + depth as i32 * 100),
    ///     }
    /// }
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter(&root);
    /// assert_eq!(iter.next(), Some((0, 0, &1)));
    /// assert_eq!(iter.next(), Some((1, 0, &102)));
    /// assert_eq!(iter.next(), Some((2, 0, &204)));
    /// assert_eq!(iter.next(), Some((3, 0, &308)));
    /// assert_eq!(iter.next(), Some((2, 1, &-205)));
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = dfs.iter(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 1, 2, 2]);
    ///
    /// let values: Vec<_> = dfs.iter(&n3).map(|x| *x.2).collect();
    /// assert_eq!(values, [-103, 206, 309, -207, 310, -311]);
    /// ```
    pub fn over_depth_sibling_data<V: TreeVariant>() -> DfsOverDepthSiblingData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the tuple of depths, sibling indices and nodes.
    ///
    /// * Iterator item => (`depth`, `sibling_idx`, [`Node`]) tuple where:
    ///   * depth is the of the node starting from the node that the iterator is created from (root) which has a depth of 0.
    ///   * sibling index of the node is its position among its siblings (or children of its parent); root's sibling index is always 0.
    ///
    /// A depth first search requires a stack to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::dfs_over`] method.
    /// However, each time the 'dfs_over' method is called a new stack is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same stack and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `DfsOverDepthNode<V>` which allocates the stack once on initialization;
    /// and creates iterators starting from different nodes that re-use the stack.
    ///
    /// `Dfs::over_depth_sibling_node::<V>()` is equivalent to `Dfs::over::<V, OverDepthSiblingNode>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::dfs_over`]: crate::NodeRef::dfs_over
    /// [`Node`]: crate::Node
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
    /// let mut tree = BinaryTree::<i32>::new(1);
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
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // its `iter` calls re-use the same stack
    /// let mut dfs = Dfs::over_depth_sibling_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter(&root);
    /// let (d, s, _n1) = iter.next().unwrap();
    /// assert_eq!(d, 0);
    /// assert_eq!(s, 0);
    ///
    /// let (d, s, n2) = iter.next().unwrap();
    /// assert_eq!(d, 1);
    /// assert_eq!(s, 0);
    ///
    /// let (d, s, n4) = iter.next().unwrap();
    /// assert_eq!(d, 2);
    /// assert_eq!(s, 0);
    ///
    /// assert_eq!(n4.data(), &4);
    /// assert_eq!(n4.parent(), Some(n2));
    /// assert_eq!(n4.num_children(), 1);
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = dfs.iter(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 1, 2, 2]);
    ///
    /// let sibling_indices: Vec<_> = dfs.iter(&n3).map(|x| x.1).collect();
    /// assert_eq!(sibling_indices, [0, 0, 0, 1, 0, 1]);
    ///
    /// let values: Vec<_> = dfs.iter(&n3).map(|x| *x.2.data()).collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn over_depth_sibling_node<V: TreeVariant>() -> DfsOverDepthSiblingNode<V> {
        Default::default()
    }
}

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
pub struct DfsIterable<V: TreeVariant, K: IterOver> {
    stack: Vec<K::DfsBfsQueueElement<V>>,
}

impl<V, K> Default for DfsIterable<V, K>
where
    V: TreeVariant,
    K: IterOver,
{
    fn default() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<V, K> DfsIterable<V, K>
where
    V: TreeVariant,
    K: IterOver,
{
    /// Creates a depth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter<'a, M, P>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> DfsIterOf<'a, V, K, M, P>
    where
        V: 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        DfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.stack)
    }

    /// Creates a mutable depth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_mut<'a, M, P>(
        &'a mut self,
        root: &'a mut NodeMut<'a, V, M, P>,
    ) -> DfsIterMutOf<'a, V, K, M, P>
    where
        V: 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        K: IterMutOver,
    {
        DfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.stack).into()
    }
}

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// Created iterators yield items of type:
/// * `V::Item` or [`data`] of the nodes.
///
/// [`data`]: crate::NodeRef::data
pub type DfsOverData<V> = DfsIterable<V, OverData>;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// Created iterators yield items of type:
/// * [`Node`]
///
/// [`Node`]: crate::Node
pub type DfsOverNode<V> = DfsIterable<V, OverNode>;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// Created iterators yield items of type:
/// * `(usize, V::Item)`:
///   * where `V::Item` is the [`data`] of the nodes,
///   * and the first item of the tuple is the depth of the nodes relative to the root.
///
/// [`data`]: crate::NodeRef::data
pub type DfsOverDepthData<V> = DfsIterable<V, OverDepthData>;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// Created iterators yield items of type:
/// * `(usize, Node)`:
///   * where the second item of the tuple is the [`Node`] itself,
///   * and the first item of the tuple is the depth of the nodes relative to the root.
///
/// [`Node`]: crate::Node
pub type DfsOverDepthNode<V> = DfsIterable<V, OverDepthNode>;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// Created iterators yield items of type:
/// * `(usize, usize, V::Item)`:
///   * where `V::Item` is the [`data`] of the nodes,
///   * the first item of the tuple is the depth of the nodes relative to the root,
///   * and the second item of the tuple is index of the nodes among its siblings.
///
/// [`data`]: crate::NodeRef::data
pub type DfsOverDepthSiblingData<V> = DfsIterable<V, OverDepthSiblingData>;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// Created iterators yield items of type:
/// * `(usize, usize, Node)`:
///   * where the third item of the tuple is the [`Node`] itself,
///   * the first item of the tuple is the depth of the nodes relative to the root,
///   * and the second item of the tuple is index of the nodes among its siblings.
///
/// [`Node`]: crate::Node
pub type DfsOverDepthSiblingNode<V> = DfsIterable<V, OverDepthSiblingNode>;

// type simplification of iterators

type DfsIterOf<'a, V, K, M, P> = DfsIter<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut Vec<<K as IterOver>::DfsBfsQueueElement<V>>,
>;

type DfsIterMutOf<'a, V, K, M, P> = DfsIterMut<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut Vec<<K as IterOver>::DfsBfsQueueElement<V>>,
>;
