use crate::{
    helpers::N,
    iter::{BfsIter, BfsIterMut, DfsIter, DfsIterMut, IterMutOver, NodeVal, NodeValueData},
    node_ref::NodeRefCore,
    tree::{DefaultMemory, DefaultPinVec},
    tree_variant::RefsChildren,
    TreeVariant,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodeIdx, NodePtr, SelfRefCol};

pub trait NodeMutOrientation {}

pub struct NodeMutDown {}
impl NodeMutOrientation for NodeMutDown {}

pub struct NodeMutUpAndDown {}
impl NodeMutOrientation for NodeMutUpAndDown {}

/// A node of the tree, which in turn is a tree.
pub struct NodeMut<'a, V, M = DefaultMemory<V>, P = DefaultPinVec<V>, O = NodeMutUpAndDown>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    O: NodeMutOrientation,
{
    col: &'a mut SelfRefCol<V, M, P>,
    node_ptr: NodePtr<V>,
    phantom: PhantomData<O>,
}

impl<'a, V, M, P, O> NodeRefCore<'a, V, M, P> for NodeMut<'a, V, M, P, O>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    O: NodeMutOrientation,
{
    #[inline(always)]
    fn col(&self) -> &SelfRefCol<V, M, P> {
        self.col
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.node_ptr
    }
}

impl<'a, V, M, P, O> NodeMut<'a, V, M, P, O>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    O: NodeMutOrientation,
{
    /// Returns a mutable reference to data of the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut().unwrap();
    ///
    /// *root.data_mut() = 10;
    /// assert_eq!(root.data(), &10);
    ///
    /// let [idx_a] = root.grow([1]);
    /// let mut node = idx_a.node_mut(&mut tree);
    ///
    /// *node.data_mut() += 10;
    /// assert_eq!(node.data(), &11);
    /// ```
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    pub fn data_mut(&mut self) -> &mut V::Item {
        self.node_mut()
            .data_mut()
            .expect("node holding a tree reference must be active")
    }

    // growth

    /// Pushes the node with the given `child` as a child of this node.
    ///
    /// If the corresponding node index of the child is required;
    /// you may use [`grow`], [`grow_iter`] or [`grow_vec`].
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`grow_iter`]: crate::NodeMut::grow_iter
    /// [`grow_vec`]: crate::NodeMut::grow_vec
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<char>::new('a');
    ///
    /// let mut node = tree.root_mut().unwrap();
    /// node.push('b');
    /// node.push('c');
    ///
    /// let mut node = node.into_child_mut(0).unwrap();
    /// node.push('d');
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, ['a', 'b', 'c', 'd']);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, ['a', 'b', 'd', 'c']);
    /// ```
    pub fn push(&mut self, child: V::Item) {
        let parent_ptr = self.node_ptr.clone();

        let child_ptr = self.col.push(child);

        let child = self.col.node_mut(&child_ptr);
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = self.col.node_mut(&parent_ptr);
        parent.next_mut().push(child_ptr.clone());
    }

    /// Pushes nodes with given `values` as children of this node.
    ///
    /// If the corresponding node indices of the children are required;
    /// you may use [`grow`]:
    ///
    /// * `node.push(child);`
    /// * `let child_idx = node.grow([child]);`
    ///
    /// [`grow`]: crate::NodeMut::grow
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<char>::new('a');
    ///
    /// let mut node = tree.root_mut().unwrap();
    /// let b = node.push('b'); // b is the index of the node
    /// node.extend(['c', 'd', 'e']);
    ///
    /// assert_eq!(node.num_children(), 4);
    /// ```
    pub fn extend<I>(&mut self, values: I)
    where
        I: IntoIterator<Item = V::Item>,
    {
        for x in values.into_iter() {
            _ = self.push(x);
        }
    }

    /// Pushes the given `children` values to children collection of this node.
    ///
    /// Returns the array node indices corresponding to each child node.
    ///
    /// See [`grow_iter`] and [`grow_vec`] to push **non-const** number of children and obtain corresponding
    /// node indices.
    ///
    /// As the name suggests, `grow`, `grow_vec` and `grow_iter` methods are convenient for building trees
    /// from top to bottom since they immediately return the indices providing access to child
    /// nodes.
    ///
    /// On the other hand, when the node indices are not required, you may use [`push`] or [`extend`] instead.
    ///
    /// [`push`]: crate::NodeMut::push
    /// [`extend`]: crate::NodeMut::extend
    /// [`grow_iter`]: crate::NodeMut::grow_iter
    /// [`grow_vec`]: crate::NodeMut::grow_vec
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
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    ///
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
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    /// ```
    pub fn grow<const N: usize>(&mut self, children: [V::Item; N]) -> [NodeIdx<V>; N] {
        children.map(|child| {
            let child_ptr = self.push_get_ptr(child);
            NodeIdx::new(self.col.memory_state(), &child_ptr)
        })
    }

    /// Pushes the given `children` values to children collection of this node.
    ///
    /// Returns the indices of the created child nodes.
    ///
    /// Note that this method returns a lazy iterator.
    /// Unless the iterator is consumed, the nodes will not be pushed to the tree.
    ///
    /// See [`grow`] when pushing a **const** number of children;
    /// and [`grow_vec`] which is a shorthand for the common use case of `node.grow_iter(children).collect::<Vec<_>>()`.
    ///
    /// As the name suggests, `grow`, `grow_vec` and `grow_iter` methods are convenient for building trees
    /// from top to bottom since they immediately return the indices providing access to child nodes.
    ///
    /// On the other hand, when the node indices are not required, you may use [`push`] or [`extend`] instead.
    ///
    /// [`push`]: crate::NodeMut::push
    /// [`extend`]: crate::NodeMut::extend
    /// [`grow`]: crate::NodeMut::grow
    /// [`grow_vec`]: crate::NodeMut::grow_vec
    ///
    /// # Examples
    ///
    /// Following example demonstrates one way to build an arbitrary depth tree with a special data structure systematically
    /// using the `grow_iter` method.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //       1          // depth: 0
    /// //      ╱ ╲
    /// //     ╱   ╲
    /// //    ╱     ╲
    /// //   3       4      // depth: 1
    /// //  ╱ ╲     ╱ ╲
    /// // 7   8   8   9    // depth: 2
    ///
    /// fn val(parent_value: i32, depth: usize, sibling_idx: usize) -> i32 {
    ///     parent_value + (depth * 2 + sibling_idx) as i32
    /// }
    ///
    /// let mut tree = DynTree::<_>::new(1);
    /// let mut idx = vec![];
    /// let mut depth_idx_range = vec![];
    ///
    /// idx.push(tree.root().unwrap().idx());
    /// depth_idx_range.push(0..1);
    ///
    /// for depth in 1..=2 {
    ///     let begin_num_nodes = idx.len();
    ///     let parent_indices: Vec<_> = depth_idx_range[depth - 1]
    ///         .clone()
    ///         .map(|x| idx[x].clone())
    ///         .collect();
    ///
    ///     for parent_idx in parent_indices {
    ///         let mut parent = parent_idx.node_mut(&mut tree);
    ///         let parent_value = *parent.data();
    ///
    ///         let children = (0..2).map(|sibling_idx| val(parent_value, depth, sibling_idx));
    ///         let children_indices = parent.grow_iter(children);
    ///         idx.extend(children_indices);
    ///     }
    ///
    ///     let end_num_nodes = idx.len();
    ///     depth_idx_range.push(begin_num_nodes..end_num_nodes);
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, [1, 3, 4, 7, 8, 8, 9]);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, [1, 3, 7, 8, 4, 8, 9]);
    /// ```
    pub fn grow_iter<'b, I>(
        &'b mut self,
        children: I,
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P, O>
    where
        I: IntoIterator<Item = V::Item>,
        I::IntoIter: 'b,
    {
        children.into_iter().map(|value| {
            let child_ptr = self.push_get_ptr(value);
            NodeIdx::new(self.col.memory_state(), &child_ptr)
        })
    }

    /// Pushes the given `children` values to children collection of this node.
    ///
    /// Returns the indices of the created child nodes collected in a vector.
    ///
    /// See [`grow`] when pushing a **const** number of children;
    /// and [`grow_iter`] for the lazy iterator variant:
    /// * `grow_vec` is a shorthand for the common use case of `node.grow_iter(children).collect::<Vec<_>>()`.
    ///
    /// As the name suggests, `grow`, `grow_vec` and `grow_iter` methods are convenient for building trees
    /// from top to bottom since they immediately return the indices providing access to child nodes.
    ///
    /// On the other hand, when the node indices are not required, you may use [`push`] or [`extend`] instead.
    ///
    /// [`push`]: crate::NodeMut::push
    /// [`extend`]: crate::NodeMut::extend
    /// [`grow`]: crate::NodeMut::grow
    /// [`grow_iter`]: crate::NodeMut::grow_iter
    ///
    /// # Examples
    ///
    /// Following example demonstrates one way to build a tree in a depth-first manner.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //       1
    /// //      ╱ ╲
    /// //     ╱   ╲
    /// //    ╱     ╲
    /// //   2       3
    /// //  ╱ ╲    ╱ | ╲
    /// // 3   4  4  5  6
    /// // |   |  |  |  |
    /// // 6   7  7  8  9
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    ///
    /// let idx_depth1 = root.grow_vec(vec![2, 3]);
    /// for idx in idx_depth1 {
    ///     let mut node = idx.node_mut(&mut tree);
    ///
    ///     let val = *node.data();
    ///     let children = (0..val).map(|x| x + 1 + val);
    ///
    ///     let idx_depth2 = node.grow_vec(children);
    ///
    ///     for idx in idx_depth2 {
    ///         let mut node = idx.node_mut(&mut tree);
    ///         node.push(*node.data() + 3);
    ///     }
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 3, 4, 4, 5, 6, 6, 7, 7, 8, 9]);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, [1, 2, 3, 6, 4, 7, 3, 4, 7, 5, 8, 6, 9]);
    /// ```
    pub fn grow_vec<I>(&mut self, children: I) -> Vec<NodeIdx<V>>
    where
        I: IntoIterator<Item = V::Item>,
    {
        self.grow_iter(children).collect()
    }

    // traversal

    /// Returns the mutable node of the `child-index`-th child of this node;
    /// returns None if the child index is out of bounds.
    ///
    /// See also [`into_child_mut`] for consuming traversal.
    ///
    /// [`into_child_mut`]: crate::NodeMut::into_child_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //       1
    /// //      ╱ ╲
    /// //     ╱   ╲
    /// //    ╱     ╲
    /// //   2       3
    /// //  ╱ ╲    ╱ | ╲
    /// // 3   4  4  5  6
    /// // |   |  |  |  |
    /// // 6   7  7  8  9
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.extend([2, 3]);
    ///
    /// for c in 0..root.num_children() {
    ///     let mut node = root.child_mut(c).unwrap();
    ///
    ///     let val = *node.data();
    ///     let children = (0..val).map(|x| x + 1 + val);
    ///
    ///     node.extend(children);
    ///
    ///     for c in 0..node.num_children() {
    ///         let mut node = node.child_mut(c).unwrap();
    ///         node.push(*node.data() + 3);
    ///     }
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 3, 4, 4, 5, 6, 6, 7, 7, 8, 9]);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, [1, 2, 3, 6, 4, 7, 3, 4, 7, 5, 8, 6, 9]);
    /// ```
    pub fn child_mut(&mut self, child_index: usize) -> Option<NodeMut<'_, V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
            .cloned()
            .map(move |p| NodeMut::new(self.col, p))
    }

    /// Consumes this mutable node and returns the mutable node of the `child-index`-th child;
    /// returns None if the child index is out of bounds.
    ///
    /// See also [`child_mut`] for non-consuming access.
    ///
    /// [`child_mut`]: crate::NodeMut::child_mut
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the following tree using into_child_mut and parent_mut:
    /// // r
    /// // |-- a
    /// //     |-- c, d, e
    /// // |-- b
    /// //     |-- f, g
    /// //            |-- h, i, j
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.extend(['a', 'b']);
    ///
    /// let mut a = root.into_child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = a.parent_mut().unwrap().into_child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// let mut g = b.into_child_mut(1).unwrap();
    /// g.extend(['h', 'i', 'j']);
    ///
    /// // check data - breadth first
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut data = vec![*root.data()]; // depth 0
    ///
    /// data.extend(root.children().map(|x| *x.data())); // depth 1
    ///
    /// for node in root.children() {
    ///     data.extend(node.children().map(|x| *x.data())); // depth 2
    /// }
    ///
    /// for node in root.children() {
    ///     for node in node.children() {
    ///         data.extend(node.children().map(|x| *x.data())); // depth 3
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     data,
    ///     ['r', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']
    /// )
    /// ```
    pub fn into_child_mut(self, child_index: usize) -> Option<NodeMut<'a, V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }

    pub fn children_mut(
        &'a mut self,
    ) -> impl ExactSizeIterator<Item = NodeMut<'a, V, M, P, NodeMutDown>> {
        let children_ptr = self.node().next().children_ptr();
        children_ptr.map(|ptr| {
            let col_mut = unsafe {
                &mut *(self.col as *const SelfRefCol<V, M, P> as *mut SelfRefCol<V, M, P>)
            };
            NodeMut::<'a, V, M, P, NodeMutDown>::new(col_mut, ptr.clone())
        })
    }

    // dfs

    /// Creates a mutable depth first search iterator over the data of the nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` which yields [`data_mut`] of each traversed node.
    ///
    /// See also [`dfs_mut_over`] for variants yielding different values for each traversed node.
    ///
    /// # Allocation
    ///
    /// Note that depth first search requires a stack (Vec) to be allocated.
    /// Each time this method is called, a stack is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use [`Dfs`] to optimize performance, which will create only the stack only once
    /// and re-use it to create many iterators.
    ///
    /// [`data_mut`]: crate::NodeMut::data_mut
    /// [`dfs_mut_over`]: crate::NodeMut::dfs_mut_over
    /// [`Dfs`]: crate::Dfs
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
    /// let mut n2 = root.into_child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.into_child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
    /// let n3_children_idx: Vec<_> = n3.grow_iter([6, 7]).collect();
    ///
    /// let mut n6 = n3.into_child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().into_child_mut(1).unwrap();
    /// n7.extend([10, 11]);
    ///
    /// // depth-first-search (dfs) from the root
    ///
    /// for x in tree.root_mut().unwrap().dfs_mut() {
    ///     *x *= 10;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(values, [10, 20, 40, 80, 50, 30, 60, 90, 70, 100, 110]);
    ///
    /// // dfs from any node
    ///
    /// let n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
    /// for x in n3.dfs_mut() {
    ///     *x /= 10;
    /// }
    /// let values: Vec<_> = n3.dfs().copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    ///
    /// let mut n6 = n3_children_idx[0].node_mut(&mut tree);
    /// for x in n6.dfs_mut() {
    ///     *x *= 100;
    /// }
    /// let values: Vec<_> = n6.dfs().copied().collect();
    /// assert_eq!(values, [600, 900]);
    ///
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(values, [10, 20, 40, 80, 50, 3, 600, 900, 7, 10, 11]);
    /// ```
    pub fn dfs_mut(&self) -> DfsIterMut<NodeVal<NodeValueData>, V, M, P> {
        DfsIter::new(self.col(), self.node_ptr().clone()).into()
    }

    /// Creates a mutable depth first search iterator over different values of nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic type parameter:
    ///
    /// * [`OverData`] yields data_mut of nodes (therefore, node.dfs_mut_over::&lt;Data&gt;() is equivalent to node.dfs_mut())
    /// * [`OverDepthData`] yields (depth, data_mut) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data_mut) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`data_mut`]: crate::NodeRef::data_mut
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// # Allocation
    ///
    /// Note that depth first search requires a stack (Vec) to be allocated.
    /// Each time this method is called, a stack is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use [`Dfs`] to optimize performance, which will create only the stack only once
    /// and re-use it to create many iterators.
    ///
    /// [`Dfs`]: crate::Dfs
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::iter::*;
    ///
    /// fn init_tree() -> DynTree<i32> {
    ///     //      1
    ///     //     ╱ ╲
    ///     //    ╱   ╲
    ///     //   2     3
    ///     //  ╱ ╲   ╱ ╲
    ///     // 4   5 6   7
    ///     // |     |  ╱ ╲
    ///     // 8     9 10  11
    ///     let mut tree = DynTree::<i32>::new(1);
    ///
    ///     let mut root = tree.root_mut().unwrap();
    ///     root.extend([2, 3]);
    ///
    ///     let mut n2 = root.into_child_mut(0).unwrap();
    ///     n2.extend([4, 5]);
    ///
    ///     let mut n4 = n2.into_child_mut(0).unwrap();
    ///     n4.push(8);
    ///
    ///     let mut n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
    ///     n3.extend([6, 7]);
    ///
    ///     let mut n6 = n3.into_child_mut(0).unwrap();
    ///     n6.push(9);
    ///
    ///     let mut n7 = n6.parent_mut().unwrap().into_child_mut(1).unwrap();
    ///     n7.extend([10, 11]);
    ///
    ///     tree
    /// }
    ///
    /// // dfs over data_mut
    ///
    /// let mut tree = init_tree();
    ///
    /// let root = tree.root_mut().unwrap();
    ///
    /// // equivalent to `root.dfs_mut()`
    /// for data in root.dfs_mut_over::<OverData>() {
    ///     *data += 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [101, 102, 104, 108, 105, 103, 106, 109, 107, 110, 111]
    /// );
    ///
    /// // dfs over (depth, data_mut)
    ///
    /// let mut tree = init_tree();
    ///
    /// let root = tree.root_mut().unwrap();
    ///
    /// for (depth, data) in root.dfs_mut_over::<OverDepthData>() {
    ///     *data += depth as i32 * 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [1, 102, 204, 308, 205, 103, 206, 309, 207, 310, 311]
    /// );
    ///
    /// // dfs over (depth, sibling index, data_mut)
    ///
    /// let mut tree = init_tree();
    ///
    /// let root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, data) in root.dfs_mut_over::<OverDepthSiblingData>() {
    ///     *data += depth as i32 * 100 + sibling_idx as i32 * 10000;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [1, 102, 204, 308, 10205, 10103, 206, 309, 10207, 310, 10311]
    /// );
    /// ```
    pub fn dfs_mut_over<K: IterMutOver>(
        &'a self,
    ) -> DfsIterMut<'a, K::IterKind<'a, V, M, P>, V, M, P> {
        DfsIter::new(self.col(), self.node_ptr().clone()).into()
    }

    // bfs

    /// Creates a mutable breadth first search iterator over the data of the nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` which yields [`data_mut`] of each traversed node.
    ///
    /// See also [`bfs_mut_over`] for variants yielding different values for each traversed node.
    ///
    /// [`data_mut`]: crate::NodeMut::data_mut
    /// [`bfs_mut_over`]: crate::NodeMut::bfs_mut_over
    ///
    /// # Allocation
    ///
    /// Note that breadth first search requires a queue (VecDeque) to be allocated.
    /// Each time this method is called, a queue is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use [`Bfs`] to optimize performance, which will create only the queue only once
    /// and re-use it to create many iterators.
    ///
    /// [`Bfs`]: crate::Bfs
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
    /// let mut n2 = root.into_child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.into_child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
    /// let n3_children_idx: Vec<_> = n3.grow_iter([6, 7]).collect();
    ///
    /// let mut n6 = n3.into_child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().into_child_mut(1).unwrap();
    /// n7.extend([10, 11]);
    ///
    /// // depth-first-search (dfs) from the root
    ///
    /// for x in tree.root_mut().unwrap().bfs_mut() {
    ///     *x *= 10;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(values, [10, 20, 30, 40, 50, 60, 70, 80, 90, 100, 110]);
    ///
    /// // bfs from any node
    ///
    /// let n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
    /// for x in n3.bfs_mut() {
    ///     *x /= 10;
    /// }
    /// let values: Vec<_> = n3.bfs().copied().collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    ///
    /// let mut n6 = n3_children_idx[0].node_mut(&mut tree);
    /// for x in n6.bfs_mut() {
    ///     *x *= 100;
    /// }
    /// let values: Vec<_> = n6.bfs().copied().collect();
    /// assert_eq!(values, [600, 900]);
    ///
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(values, [10, 20, 3, 40, 50, 600, 7, 80, 900, 10, 11]);
    /// ```
    pub fn bfs_mut(&self) -> BfsIterMut<NodeVal<NodeValueData>, V, M, P> {
        BfsIter::new(self.col(), self.node_ptr().clone()).into()
    }

    /// Creates a mutable breadth first search iterator over different values of nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic type parameter:
    ///
    /// * [`OverData`] yields data_mut of nodes (therefore, node.dfs_mut_over::&lt;Data&gt;() is equivalent to node.dfs_mut())
    /// * [`OverDepthData`] yields (depth, data_mut) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data_mut) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`data_mut`]: crate::NodeRef::data_mut
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// # Allocation
    ///
    /// Note that breadth first search requires a queue (VecDeque) to be allocated.
    /// Each time this method is called, a queue is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use [`Bfs`] to optimize performance, which will create only the queue only once
    /// and re-use it to create many iterators.
    ///
    /// [`Bfs`]: crate::Bfs
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::iter::*;
    ///
    /// fn init_tree() -> DynTree<i32> {
    ///     //      1
    ///     //     ╱ ╲
    ///     //    ╱   ╲
    ///     //   2     3
    ///     //  ╱ ╲   ╱ ╲
    ///     // 4   5 6   7
    ///     // |     |  ╱ ╲
    ///     // 8     9 10  11
    ///     let mut tree = DynTree::<i32>::new(1);
    ///
    ///     let mut root = tree.root_mut().unwrap();
    ///     root.extend([2, 3]);
    ///
    ///     let mut n2 = root.into_child_mut(0).unwrap();
    ///     n2.extend([4, 5]);
    ///
    ///     let mut n4 = n2.into_child_mut(0).unwrap();
    ///     n4.push(8);
    ///
    ///     let mut n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
    ///     n3.extend([6, 7]);
    ///
    ///     let mut n6 = n3.into_child_mut(0).unwrap();
    ///     n6.push(9);
    ///
    ///     let mut n7 = n6.parent_mut().unwrap().into_child_mut(1).unwrap();
    ///     n7.extend([10, 11]);
    ///
    ///     tree
    /// }
    ///
    /// // bfs over data_mut
    ///
    /// let mut tree = init_tree();
    ///
    /// let root = tree.root_mut().unwrap();
    ///
    /// // equivalent to `root.bfs_mut()`
    /// for data in root.bfs_mut_over::<OverData>() {
    ///     *data += 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111]
    /// );
    ///
    /// // bfs over (depth, data_mut)
    ///
    /// let mut tree = init_tree();
    ///
    /// let root = tree.root_mut().unwrap();
    ///
    /// for (depth, data) in root.bfs_mut_over::<OverDepthData>() {
    ///     *data += depth as i32 * 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [1, 102, 103, 204, 205, 206, 207, 308, 309, 310, 311]
    /// );
    ///
    /// // bfs over (depth, sibling index, data_mut)
    ///
    /// let mut tree = init_tree();
    ///
    /// let root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, data) in root.bfs_mut_over::<OverDepthSiblingData>() {
    ///     *data += depth as i32 * 100 + sibling_idx as i32 * 10000;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [1, 102, 10103, 204, 10205, 206, 10207, 308, 309, 310, 10311]
    /// );
    /// ```
    pub fn bfs_mut_over<K: IterMutOver>(
        &'a self,
    ) -> BfsIterMut<'a, K::IterKind<'a, V, M, P>, V, M, P> {
        BfsIter::new(self.col(), self.node_ptr().clone()).into()
    }

    // helpers

    pub(crate) fn new(col: &'a mut SelfRefCol<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Self {
            col,
            node_ptr,
            phantom: PhantomData,
        }
    }

    fn node_mut(&mut self) -> &mut N<V> {
        unsafe { &mut *self.node_ptr().ptr_mut() }
    }

    pub(crate) fn push_get_ptr(&mut self, value: V::Item) -> NodePtr<V> {
        let parent_ptr = self.node_ptr.clone();

        let child_ptr = self.col.push(value);

        let child = self.col.node_mut(&child_ptr);
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = self.col.node_mut(&parent_ptr);
        parent.next_mut().push(child_ptr.clone());

        child_ptr
    }
}

impl<'a, V, M, P> NodeMut<'a, V, M, P, NodeMutUpAndDown>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Consumes this mutable node and returns the mutable node of its parent,
    /// returns None if this is the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the following tree using into_child_mut and parent_mut:
    /// // r
    /// // |-- a
    /// //     |-- c, d, e
    /// // |-- b
    /// //     |-- f, g
    /// //            |-- h, i, j
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.extend(['a', 'b']);
    ///
    /// let mut a = root.into_child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = a.parent_mut().unwrap().into_child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// let mut g = b.into_child_mut(1).unwrap();
    /// g.extend(['h', 'i', 'j']);
    ///
    /// // check data - breadth first
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut data = vec![*root.data()]; // depth 0
    ///
    /// data.extend(root.children().map(|x| *x.data())); // depth 1
    ///
    /// for node in root.children() {
    ///     data.extend(node.children().map(|x| *x.data())); // depth 2
    /// }
    ///
    /// for node in root.children() {
    ///     for node in node.children() {
    ///         data.extend(node.children().map(|x| *x.data())); // depth 3
    ///     }
    /// }
    ///
    /// assert_eq!(
    ///     data,
    ///     ['r', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']
    /// )
    /// ```
    pub fn parent_mut(self) -> Option<NodeMut<'a, V, M, P>> {
        self.node()
            .prev()
            .get()
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }
}

#[test]
fn abc() {
    use super::*;
    use crate::iter::*;
    use crate::*;
    use alloc::vec;
    use alloc::vec::Vec;

    //       1          // depth: 0
    //      ╱ ╲
    //     ╱   ╲
    //    ╱     ╲
    //   3       4      // depth: 1
    //  ╱ ╲     ╱ ╲
    // 7   8   8   9    // depth: 2

    fn val(parent_value: i32, depth: usize, sibling_idx: usize) -> i32 {
        parent_value + (depth * 2 + sibling_idx) as i32
    }

    let mut tree = DynTree::<_>::new(1);
    let mut idx = vec![];
    let mut depth_idx_range = vec![];

    idx.push(tree.root().unwrap().idx());
    depth_idx_range.push(0..1);

    for depth in [1, 2] {
        let begin_num_nodes = idx.len();
        let parent_indices: Vec<_> = depth_idx_range[depth - 1]
            .clone()
            .map(|x| idx[x].clone())
            .collect();

        for parent_idx in parent_indices {
            let mut parent = parent_idx.node_mut(&mut tree);
            let parent_value = *parent.data();

            let children = (0..2).map(|sibling_idx| val(parent_value, depth, sibling_idx));
            let children_indices = parent.grow_iter(children);
            idx.extend(children_indices);
        }

        let end_num_nodes = idx.len();
        depth_idx_range.push(begin_num_nodes..end_num_nodes);
    }

    // validate the tree

    let root = tree.root().unwrap();

    let bfs: Vec<_> = root.bfs().copied().collect();
    assert_eq!(bfs, [1, 3, 4, 7, 8, 8, 9]);

    let dfs: Vec<_> = root.dfs().copied().collect();
    assert_eq!(dfs, [1, 3, 7, 8, 4, 8, 9]);
}
