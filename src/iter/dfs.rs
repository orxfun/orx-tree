use super::{IterOver, OverData, OverDepthData, OverDepthNode, OverNode};
use crate::{helpers::N, iter::DfsIter, NodeRef, TreeVariant};
use alloc::vec::Vec;
use orx_selfref_col::MemoryPolicy;
use orx_split_vec::PinnedVec;

pub struct Dfs;

impl Dfs {
    pub fn over<K: IterOver, V: TreeVariant>() -> DfsCore<V, K> {
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
    /// root.extend([2, 3]);
    ///
    /// let mut n2 = root.child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// n3.extend([6, 7]);
    ///
    /// let mut n6 = n3.child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
    /// n7.extend([10, 11]);
    ///
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // succeeding `iter_from` calls re-use the same stack
    /// let mut dfs = Dfs::over_data();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter_from(&root);
    /// assert_eq!(iter.next(), Some(&1));
    /// assert_eq!(iter.next(), Some(&2));
    /// assert_eq!(iter.next(), Some(&4));
    /// assert_eq!(iter.next(), Some(&8));
    /// assert_eq!(iter.next(), Some(&5)); // ...
    ///
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = dfs.iter_from(&n3).copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
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
    /// root.extend([2, 3]);
    ///
    /// let mut n2 = root.child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// n3.extend([6, 7]);
    ///
    /// let mut n6 = n3.child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
    /// n7.extend([10, 11]);
    ///
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // succeeding `iter_from` calls re-use the same stack
    /// let mut dfs = Dfs::over_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter_from(&root);
    /// let _n1 = iter.next().unwrap();
    /// let n2 = iter.next().unwrap();
    /// let n4 = iter.next().unwrap();
    /// assert_eq!(n4.data(), &4);
    /// assert_eq!(n4.parent(), Some(n2));
    /// assert_eq!(n4.num_children(), 1);
    ///
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = dfs.iter_from(&n3).map(|x| *x.data()).collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn over_node<V: TreeVariant>() -> DfsOverNode<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the tuple of depths and nodes.
    ///
    /// * Iterator item => (`depth`, [`data`]) tuple where depth of the node that the iterator is set to zero (the root).
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
    /// root.extend([2, 3]);
    ///
    /// let mut n2 = root.child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// n3.extend([6, 7]);
    ///
    /// let mut n6 = n3.child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
    /// n7.extend([10, 11]);
    ///
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // succeeding `iter_from` calls re-use the same stack
    /// let mut dfs = Dfs::over_depth_data();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter_from(&root);
    /// assert_eq!(iter.next(), Some((0, &1)));
    /// assert_eq!(iter.next(), Some((1, &2)));
    /// assert_eq!(iter.next(), Some((2, &4)));
    /// assert_eq!(iter.next(), Some((3, &8)));
    /// assert_eq!(iter.next(), Some((2, &5)));
    ///
    /// let n3 = root.child(1).unwrap();
    ///
    /// let depths: Vec<_> = dfs.iter_from(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 1, 2, 2]);
    ///
    /// let values: Vec<_> = dfs.iter_from(&n3).map(|x| *x.1).collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn over_depth_data<V: TreeVariant>() -> DfsOverDepthData<V> {
        Default::default()
    }

    /// Creates an iterable, which can be used to create depth-first iterators over the tuple of depths and nodes.
    ///
    /// * Iterator item => (`depth`, [`Node`]) tuple where depth of the node that the iterator is set to zero (the root).
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
    /// root.extend([2, 3]);
    ///
    /// let mut n2 = root.child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// n3.extend([6, 7]);
    ///
    /// let mut n6 = n3.child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
    /// n7.extend([10, 11]);
    ///
    /// // create re-usable dfs iterable
    /// // stack is created here, only once
    /// // succeeding `iter_from` calls re-use the same stack
    /// let mut dfs = Dfs::over_depth_node();
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = dfs.iter_from(&root);
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
    /// let depths: Vec<_> = dfs.iter_from(&n3).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 1, 2, 2]);
    ///
    /// let values: Vec<_> = dfs.iter_from(&n3).map(|x| *x.1.data()).collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn over_depth_node<V: TreeVariant>() -> DfsOverDepthNode<V> {
        Default::default()
    }
}

pub struct DfsCore<V: TreeVariant, K: IterOver> {
    stack: Vec<K::QueueElement<V>>,
}

impl<V, K> Default for DfsCore<V, K>
where
    V: TreeVariant,
    K: IterOver,
{
    fn default() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<V, K> DfsCore<V, K>
where
    V: TreeVariant,
    K: IterOver,
{
    pub fn new() -> Self {
        Default::default()
    }

    pub fn iter_from<'a, M, P>(
        &'a mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> DfsIter<'a, K::IterKind<'a, V, M, P>, V, M, P, &'a mut Vec<K::QueueElement<V>>>
    where
        V: 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        DfsIter::new_with_queue(node.col(), node.node_ptr().clone(), &mut self.stack)
    }
}

pub type DfsOverData<V: TreeVariant> = DfsCore<V, OverData>;

pub type DfsOverNode<V: TreeVariant> = DfsCore<V, OverNode>;

pub type DfsOverDepthData<V: TreeVariant> = DfsCore<V, OverDepthData>;

pub type DfsOverDepthNode<V: TreeVariant> = DfsCore<V, OverDepthNode>;

#[test]
fn abc() {
    use crate::iter::*;
    use crate::*;
    use alloc::vec;
    use alloc::vec::Vec;

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11
    let mut tree = BinaryTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();
    root.extend([2, 3]);

    let mut n2 = root.child_mut(0).unwrap();
    n2.extend([4, 5]);

    let mut n4 = n2.child_mut(0).unwrap();
    n4.push(8);

    let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    n3.extend([6, 7]);

    let mut n6 = n3.child_mut(0).unwrap();
    n6.push(9);

    let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
    n7.extend([10, 11]);

    // create re-usable dfs iterable
    // stack is created here, only once
    // succeeding `iter_from` calls re-use the same stack
    let mut dfs = Dfs::over_depth_data();

    let root = tree.root().unwrap();

    let mut iter = dfs.iter_from(&root);
    assert_eq!(iter.next(), Some((0, &1)));
    assert_eq!(iter.next(), Some((1, &2)));
    assert_eq!(iter.next(), Some((2, &4)));
    assert_eq!(iter.next(), Some((3, &8)));
    assert_eq!(iter.next(), Some((2, &5)));

    let n3 = root.child(1).unwrap();

    let depths: Vec<_> = dfs.iter_from(&n3).map(|x| x.0).collect();
    assert_eq!(depths, [0, 1, 2, 1, 2, 2]);

    let values: Vec<_> = dfs.iter_from(&n3).map(|x| *x.1).collect();
    assert_eq!(values, [3, 6, 9, 7, 10, 11]);
}

#[test]
fn def() {
    use crate::iter::*;
    use crate::*;
    use alloc::vec;
    use alloc::vec::Vec;

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11
    let mut tree = BinaryTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();
    root.extend([2, 3]);

    let mut n2 = root.child_mut(0).unwrap();
    n2.extend([4, 5]);

    let mut n4 = n2.child_mut(0).unwrap();
    n4.push(8);

    let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    n3.extend([6, 7]);

    let mut n6 = n3.child_mut(0).unwrap();
    n6.push(9);

    let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
    n7.extend([10, 11]);

    // create re-usable dfs iterable
    // stack is created here, only once
    // succeeding `iter_from` calls re-use the same stack
    let mut dfs = Dfs::over_depth_node();

    let root = tree.root().unwrap();

    let mut iter = dfs.iter_from(&root);
    let (d, _n1) = iter.next().unwrap();
    assert_eq!(d, 0);

    let (d, n2) = iter.next().unwrap();
    assert_eq!(d, 1);

    let (d, n4) = iter.next().unwrap();
    assert_eq!(d, 2);

    assert_eq!(n4.data(), &4);
    assert_eq!(n4.parent(), Some(n2));
    assert_eq!(n4.num_children(), 1);

    let n3 = root.child(1).unwrap();

    let depths: Vec<_> = dfs.iter_from(&n3).map(|x| x.0).collect();
    assert_eq!(depths, [0, 1, 2, 1, 2, 2]);

    let values: Vec<_> = dfs.iter_from(&n3).map(|x| *x.1.data()).collect();
    assert_eq!(values, [3, 6, 9, 7, 10, 11]);
}
