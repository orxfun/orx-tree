use super::BfsIterMut;
use crate::iter::{
    IterMutOver, IterOver, OverData, OverDepthData, OverDepthNode, OverDepthSiblingData,
    OverDepthSiblingNode, OverNode,
};
use crate::{helpers::N, iter::BfsIter, node_ref::NodeRefCore, NodeMut, NodeRef, TreeVariant};
use alloc::collections::VecDeque;
use orx_selfref_col::MemoryPolicy;
use orx_split_vec::PinnedVec;

/// Factory to create breadth-first-search iterables, which in turn can be used to create iterators rooted from different nodes.
///
/// Note that breadth-first-search iterators can directly be created:
/// * from [`NodeRef`] ([`Node`] or [`NodeMut`]) using [`bfs`] or [`bfs_over`] methods, or
/// * from [`NodeMut`] using [`bfs_mut`] or [`bfs_mut_over`] methods.
///
/// Note that the breadth first traversal requires a queue.
/// Each time an iterator is crated using above-mentioned tree node methods, a new queue (VecDeque) is allocated and dropped once the iterator is dropped.
///
/// On the other hand, iterables created from `Bfs` allocate the queue on initialization,
/// and keep re-using the same queue regardless of how many iterators are created from it.
/// This allows to minimize memory allocations and optimize performance in programs where we repeatedly traverse the tree.
///
/// [`Node`]: crate::Node
/// [`bfs`]: crate::NodeRef::bfs
/// [`bfs_over`]: crate::NodeRef::bfs_over
/// [`bfs_mut`]: crate::NodeMut::bfs_mut
/// [`bfs_mut_over`]: crate::NodeMut::bfs_mut_over
pub struct Bfs;

impl Bfs {
    /// Creates an iterable which can be used to create breadth-first iterators.
    ///
    /// Item or element type of the created iterators depend on the generic `K: IterOver` parameter:
    ///
    /// ```ignore
    /// Bfs::over::<OverData, _>() <===> Bfs::over_data::<_>()   // yields `data`
    /// Bfs::over::<OverNode, _>() <===> Bfs::over_node::<_>()   // yields `Node`
    ///
    /// Bfs::over::<OverDepthData, _>() <===> Bfs::over_depth_data::<_>()   // yields (breadth, `data`)
    /// Bfs::over::<OverDepthNode, _>() <===> Bfs::over_depth_node::<_>()   // yields (breadth, `Node`)
    ///
    /// Bfs::over::<OverDepthSiblingData, _>() <===> Bfs::over_depth_sibling_data::<_>()   // yields (breadth, sibling_idx, `data`)
    /// Bfs::over::<OverDepthSiblingNode, _>() <===> Bfs::over_depth_sibling_node::<_>()   // yields (breadth, sibling_idx, `Node`)
    /// ```
    ///
    /// Depth and sibling indices are based on the node that the created iterator is rooted at:
    /// * breadth is the of the node starting from the node that the iterator is created from (root) which has a breadth of 0.
    /// * sibling index of the node is its position among its siblings (or children of its parent); root's sibling index is always 0.
    pub fn over<K: IterOver, V: TreeVariant>() -> BfsCore<V, K> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create breadth-first iterators over [`data`] of nodes.
    ///
    /// A breadth first search requires a queue to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::bfs`] method.
    /// However, each time the 'bfs' method is called a new queue is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same queue and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `BfsOverData<V>` which allocates the queue once on initialization;
    /// and creates iterators starting from different nodes that re-use the queue.
    ///
    /// `Bfs::over_data::<V>()` is equivalent to `Bfs::over::<V, OverData>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::bfs`]: crate::NodeRef::bfs
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
    /// // create re-usable bfs iterable
    /// // queue is created here, only once
    /// // its `iter_from` and `iter_mut_from` calls re-use the same queue
    /// let mut bfs = Bfs::over_data();
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// for x in bfs.iter_mut_from(&mut root) {
    ///     *x *= 100;
    /// }
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = bfs.iter_from(&root);
    /// assert_eq!(iter.next(), Some(&100));
    /// assert_eq!(iter.next(), Some(&200));
    /// assert_eq!(iter.next(), Some(&300));
    /// assert_eq!(iter.next(), Some(&400));
    /// assert_eq!(iter.next(), Some(&500)); // ...
    ///
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = bfs.iter_from(&n3).copied().collect();
    /// assert_eq!(values, [300, 600, 700, 900, 1000, 1100]);
    /// ```
    pub fn over_data<V: TreeVariant>() -> BfsOverData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create breadth-first iterators over the nodes ([`Node`]).
    ///
    /// A breadth first search requires a queue to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::bfs_over`] method.
    /// However, each time the 'bfs_over' method is called a new queue is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same queue and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `BfsOverNode<V>` which allocates the queue once on initialization;
    /// and creates iterators starting from different nodes that re-use the queue.
    ///
    /// `Bfs::over_node::<V>()` is equivalent to `Bfs::over::<V, OverNode>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::bfs_over`]: crate::NodeRef::bfs_over
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
    /// // create re-usable bfs iterable
    /// // queue is created here, only once
    /// // its `iter_from` calls re-use the same queue
    /// let mut bfs = Bfs::over_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = bfs.iter_from(&root);
    /// let n1 = iter.next().unwrap();
    /// let _n2 = iter.next().unwrap();
    /// let n3 = iter.next().unwrap();
    /// assert_eq!(n3.data(), &3);
    /// assert_eq!(n3.parent(), Some(n1));
    /// assert_eq!(n3.num_children(), 2);
    ///
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = bfs.iter_from(&n3).map(|x| *x.data()).collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    /// ```
    pub fn over_node<V: TreeVariant>() -> BfsOverNode<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create breadth-first iterators over the tuple of depths and nodes.
    ///
    /// * Iterator item => (`breadth`, [`data`]) tuple where:
    ///   * breadth is the of the node starting from the node that the iterator is created from (root) which has a breadth of 0.
    ///
    /// A breadth first search requires a queue to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::bfs_over`] method.
    /// However, each time the 'bfs_over' method is called a new queue is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same queue and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `BfsOverDepthData<V>` which allocates the queue once on initialization;
    /// and creates iterators starting from different nodes that re-use the queue.
    ///
    /// `Bfs::over_depth_data::<V>()` is equivalent to `Bfs::over::<V, OverDepthData>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::bfs_over`]: crate::NodeRef::bfs_over
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
    /// // create re-usable bfs iterable
    /// // queue is created here, only once
    /// // its `iter_from` and `iter_mut_from` calls re-use the same queue
    /// let mut bfs = Bfs::over_depth_data();
    ///
    /// for (breadth, x) in bfs.iter_mut_from(&mut tree.root_mut().unwrap()) {
    ///     *x += breadth as i32 * 100;
    /// }
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = bfs.iter_from(&root);
    /// assert_eq!(iter.next(), Some((0, &1)));
    /// assert_eq!(iter.next(), Some((1, &102)));
    /// assert_eq!(iter.next(), Some((1, &103)));
    /// assert_eq!(iter.next(), Some((2, &204)));
    /// assert_eq!(iter.next(), Some((2, &205)));
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = bfs.iter_from(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2]);
    ///
    /// let values: Vec<_> = bfs.iter_from(&n3).map(|x| *x.1).collect();
    /// assert_eq!(values, [103, 206, 207, 309, 310, 311]);
    /// ```
    pub fn over_depth_data<V: TreeVariant>() -> BfsOverDepthData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create breadth-first iterators over the tuple of depths and nodes.
    ///
    /// * Iterator item => (`breadth`, [`Node`]) tuple where:
    ///   * breadth is the of the node starting from the node that the iterator is created from (root) which has a breadth of 0.
    ///
    /// A breadth first search requires a queue to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::bfs_over`] method.
    /// However, each time the 'bfs_over' method is called a new queue is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same queue and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `BfsOverDepthNode<V>` which allocates the queue once on initialization;
    /// and creates iterators starting from different nodes that re-use the queue.
    ///
    /// `Bfs::over_depth_node::<V>()` is equivalent to `Bfs::over::<V, OverDepthNode>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::bfs_over`]: crate::NodeRef::bfs_over
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
    /// // create re-usable bfs iterable
    /// // queue is created here, only once
    /// // its `iter_from` calls re-use the same queue
    /// let mut bfs = Bfs::over_depth_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = bfs.iter_from(&root);
    /// let (d, n1) = iter.next().unwrap();
    /// assert_eq!(d, 0);
    ///
    /// let (d, _n2) = iter.next().unwrap();
    /// assert_eq!(d, 1);
    ///
    /// let (d, n3) = iter.next().unwrap();
    /// assert_eq!(d, 1);
    ///
    /// assert_eq!(n3.data(), &3);
    /// assert_eq!(n3.parent(), Some(n1));
    /// assert_eq!(n3.num_children(), 2);
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = bfs.iter_from(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2]);
    ///
    /// let values: Vec<_> = bfs.iter_from(&n3).map(|x| *x.1.data()).collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    /// ```
    pub fn over_depth_node<V: TreeVariant>() -> BfsOverDepthNode<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create breadth-first iterators over the tuple of depths, sibling indices and nodes.
    ///
    /// * Iterator item => (`breadth`, `sibling_idx` [`data`]) tuple where:
    ///   * breadth is the of the node starting from the node that the iterator is created from (root) which has a breadth of 0.
    ///   * sibling index of the node is its position among its siblings (or children of its parent); root's sibling index is always 0.
    ///
    /// A breadth first search requires a queue to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::bfs_over`] method.
    /// However, each time the 'bfs_over' method is called a new queue is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same queue and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `BfsOverDepthData<V>` which allocates the queue once on initialization;
    /// and creates iterators starting from different nodes that re-use the queue.
    ///
    /// `Bfs::over_depth_sibling_data::<V>()` is equivalent to `Bfs::over::<V, OverDepthSiblingData>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::bfs_over`]: crate::NodeRef::bfs_over
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
    /// // create re-usable bfs iterable
    /// // queue is created here, only once
    /// // its `iter_from` and `iter_mut_from` calls re-use the same queue
    /// let mut bfs = Bfs::over_depth_sibling_data();
    ///
    /// for (breadth, sibling_idx, x) in bfs.iter_mut_from(&mut tree.root_mut().unwrap()) {
    ///     match sibling_idx {
    ///         0 => *x += breadth as i32 * 100,
    ///         _ => *x = -(*x + breadth as i32 * 100),
    ///     }
    /// }
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = bfs.iter_from(&root);
    /// assert_eq!(iter.next(), Some((0, 0, &1)));
    /// assert_eq!(iter.next(), Some((1, 0, &102)));
    /// assert_eq!(iter.next(), Some((1, 1, &-103)));
    /// assert_eq!(iter.next(), Some((2, 0, &204)));
    /// assert_eq!(iter.next(), Some((2, 1, &-205)));
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = bfs.iter_from(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2]);
    ///
    /// let values: Vec<_> = bfs.iter_from(&n3).map(|x| *x.2).collect();
    /// assert_eq!(values, [-103, 206, -207, 309, 310, -311]);
    /// ```
    pub fn over_depth_sibling_data<V: TreeVariant>() -> BfsOverDepthSiblingData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create breadth-first iterators over the tuple of depths, sibling indices and nodes.
    ///
    /// * Iterator item => (`breadth`, `sibling_idx`, [`Node`]) tuple where:
    ///   * breadth is the of the node starting from the node that the iterator is created from (root) which has a breadth of 0.
    ///   * sibling index of the node is its position among its siblings (or children of its parent); root's sibling index is always 0.
    ///
    /// A breadth first search requires a queue to be allocated.
    ///
    /// One can iterate starting from different nodes of the tree multiple times using the [`NodeRef::bfs_over`] method.
    /// However, each time the 'bfs_over' method is called a new queue is allocated and dropped once the iterator is dropped.
    /// This might not be a problem in many use cases.
    ///
    /// However, when memory is more scarce and we iterate many times over different nodes of the tree,
    /// we can re-use the same queue and limit the allocation to only one vector, regardless of how many times we iterate.
    ///
    /// This method crates an iterable `BfsOverDepthNode<V>` which allocates the queue once on initialization;
    /// and creates iterators starting from different nodes that re-use the queue.
    ///
    /// `Bfs::over_depth_sibling_node::<V>()` is equivalent to `Bfs::over::<V, OverDepthSiblingNode>()`.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`NodeRef::bfs_over`]: crate::NodeRef::bfs_over
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
    /// // create re-usable bfs iterable
    /// // queue is created here, only once
    /// // its `iter_from` calls re-use the same queue
    /// let mut bfs = Bfs::over_depth_sibling_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = bfs.iter_from(&root);
    /// let (d, s, n1) = iter.next().unwrap();
    /// assert_eq!(d, 0);
    /// assert_eq!(s, 0);
    ///
    /// let (d, s, _n2) = iter.next().unwrap();
    /// assert_eq!(d, 1);
    /// assert_eq!(s, 0);
    ///
    /// let (d, s, n3) = iter.next().unwrap();
    /// assert_eq!(d, 1);
    /// assert_eq!(s, 1);
    ///
    /// assert_eq!(n3.data(), &3);
    /// assert_eq!(n3.parent(), Some(n1));
    /// assert_eq!(n3.num_children(), 2);
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = bfs.iter_from(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2]);
    ///
    /// let sibling_indices: Vec<_> = bfs.iter_from(&n3).map(|x| x.1).collect();
    /// assert_eq!(sibling_indices, [0, 0, 1, 0, 0, 1]);
    ///
    /// let values: Vec<_> = bfs.iter_from(&n3).map(|x| *x.2.data()).collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    /// ```
    pub fn over_depth_sibling_node<V: TreeVariant>() -> BfsOverDepthSiblingNode<V> {
        Default::default()
    }
}

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
pub struct BfsCore<V: TreeVariant, K: IterOver> {
    queue: VecDeque<K::DfsBfsQueueElement<V>>,
}

impl<V, K> Default for BfsCore<V, K>
where
    V: TreeVariant,
    K: IterOver,
{
    fn default() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }
}

impl<V, K> BfsCore<V, K>
where
    V: TreeVariant,
    K: IterOver,
{
    /// Creates a breadth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the breadth-first-search will be rooted at this node:
    /// * root's breadth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_from<'a, M, P>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> BfsIterOf<'a, V, K, M, P>
    where
        V: 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        BfsIter::new_with_queue(root.col(), root.node_ptr().clone(), &mut self.queue)
    }

    /// Creates a mutable breadth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the breadth-first-search will be rooted at this node:
    /// * root's breadth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_mut_from<'a, M, P>(
        &'a mut self,
        root: &'a mut NodeMut<'a, V, M, P>,
    ) -> BfsIterMutOf<'a, V, K, M, P>
    where
        V: 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        K: IterMutOver,
    {
        BfsIter::new_with_queue(root.col(), root.node_ptr().clone(), &mut self.queue).into()
    }
}

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
///
/// Created iterators yield items of type:
/// * `V::Item` or [`data`] of the nodes.
///
/// [`data`]: crate::NodeRef::data
pub type BfsOverData<V> = BfsCore<V, OverData>;

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
///
/// Created iterators yield items of type:
/// * [`Node`]
///
/// [`Node`]: crate::Node
pub type BfsOverNode<V> = BfsCore<V, OverNode>;

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
///
/// Created iterators yield items of type:
/// * `(usize, V::Item)`:
///   * where `V::Item` is the [`data`] of the nodes,
///   * and the first item of the tuple is the breadth of the nodes relative to the root.
///
/// [`data`]: crate::NodeRef::data
pub type BfsOverDepthData<V> = BfsCore<V, OverDepthData>;

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
///
/// Created iterators yield items of type:
/// * `(usize, Node)`:
///   * where the second item of the tuple is the [`Node`] itself,
///   * and the first item of the tuple is the breadth of the nodes relative to the root.
///
/// [`Node`]: crate::Node
pub type BfsOverDepthNode<V> = BfsCore<V, OverDepthNode>;

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
///
/// Created iterators yield items of type:
/// * `(usize, usize, V::Item)`:
///   * where `V::Item` is the [`data`] of the nodes,
///   * the first item of the tuple is the breadth of the nodes relative to the root,
///   * and the second item of the tuple is index of the nodes among its siblings.
///
/// [`data`]: crate::NodeRef::data
pub type BfsOverDepthSiblingData<V> = BfsCore<V, OverDepthSiblingData>;

/// An iterable which can create breadth-first iterators over and over, using the same only-once allocated queue.
///
/// Created iterators yield items of type:
/// * `(usize, usize, Node)`:
///   * where the third item of the tuple is the [`Node`] itself,
///   * the first item of the tuple is the breadth of the nodes relative to the root,
///   * and the second item of the tuple is index of the nodes among its siblings.
///
/// [`Node`]: crate::Node
pub type BfsOverDepthSiblingNode<V> = BfsCore<V, OverDepthSiblingNode>;

// type simplification of iterators

type BfsIterOf<'a, V, K, M, P> = BfsIter<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut VecDeque<<K as IterOver>::DfsBfsQueueElement<V>>,
>;

type BfsIterMutOf<'a, V, K, M, P> = BfsIterMut<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut VecDeque<<K as IterOver>::DfsBfsQueueElement<V>>,
>;
