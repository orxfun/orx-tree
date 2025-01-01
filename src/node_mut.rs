use crate::{
    helpers::N,
    iter::{BfsIter, BfsIterMut, DfsIter, DfsIterMut, IterMutOver, NodeVal, NodeValueData},
    node_ref::NodeRefCore,
    tree::{DefaultMemory, DefaultPinVec},
    tree_variant::RefsChildren,
    TreeVariant,
};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, MemoryState, NodeIdx, NodePtr, SelfRefCol};

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
    /// you may use [`grow`]`:
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
    /// node.push('b');
    /// node.push('c');
    ///
    /// let mut node = node.child_mut(0).unwrap();
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

    /// Pushes nodes with given `children` to children collection of this node.
    ///
    /// Returns the array node indices corresponding to each child node.
    ///
    /// See [`grow_iter`] to push **non-const** number of children and obtain corresponding
    /// node indices.
    ///
    /// As the name suggests, `grow` and `grow_iter` methods are convenient for building trees
    /// from top to bottom since they immediately return the indices providing access to child
    /// nodes.
    ///
    /// On the other hand, when the node indices are not required, you may use [`push`] or [`extend`] instead.
    ///
    /// [`push`]: crate::NodeMut::push
    /// [`extend`]: crate::NodeMut::extend
    /// [`grow_iter`]: crate::NodeMut::grow_iter
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

    /// Pushes nodes with the given `values` as children of this node.
    ///
    /// Returns the indices of the created child nodes.
    ///
    /// Note that this method returns a lazy iterator.
    /// Unless the iterator is consumed, the nodes will not be pushed to the tree.
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
    /// let cde = node.extend_get_indices(['c', 'd', 'e']).collect::<Vec<_>>();
    ///
    /// assert_eq!(node.num_children(), 4);
    ///
    /// let node_d = cde[1].node(&tree);
    /// assert_eq!(node_d.data(), &'d');
    /// ```
    pub fn extend_get_indices<'b, I>(
        &'b mut self,
        values: I,
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P, O>
    where
        I: IntoIterator<Item = V::Item>,
        I::IntoIter: 'b,
    {
        values.into_iter().map(|value| {
            let child_ptr = self.push_get_ptr(value);
            NodeIdx::new(self.col.memory_state(), &child_ptr)
        })
    }

    /// Consumes this mutable node and returns the mutable node of the `child-index`-th child;
    /// returns None if the child index is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the following tree using child_mut and parent_mut:
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
    /// let mut a = root.child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = a.parent_mut().unwrap().child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// let mut g = b.child_mut(1).unwrap();
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
    pub fn child_mut(self, child_index: usize) -> Option<NodeMut<'a, V, M, P>> {
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
    /// let mut n2 = root.child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// let n3_children_idx: Vec<_> = n3.extend_get_indices([6, 7]).collect();
    ///
    /// let mut n6 = n3.child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
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
    /// let n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
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
    ///     let mut n2 = root.child_mut(0).unwrap();
    ///     n2.extend([4, 5]);
    ///
    ///     let mut n4 = n2.child_mut(0).unwrap();
    ///     n4.push(8);
    ///
    ///     let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    ///     n3.extend([6, 7]);
    ///
    ///     let mut n6 = n3.child_mut(0).unwrap();
    ///     n6.push(9);
    ///
    ///     let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
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
    /// let mut n2 = root.child_mut(0).unwrap();
    /// n2.extend([4, 5]);
    ///
    /// let mut n4 = n2.child_mut(0).unwrap();
    /// n4.push(8);
    ///
    /// let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    /// let n3_children_idx: Vec<_> = n3.extend_get_indices([6, 7]).collect();
    ///
    /// let mut n6 = n3.child_mut(0).unwrap();
    /// n6.push(9);
    ///
    /// let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
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
    /// let n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
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
    ///     let mut n2 = root.child_mut(0).unwrap();
    ///     n2.extend([4, 5]);
    ///
    ///     let mut n4 = n2.child_mut(0).unwrap();
    ///     n4.push(8);
    ///
    ///     let mut n3 = tree.root_mut().unwrap().child_mut(1).unwrap();
    ///     n3.extend([6, 7]);
    ///
    ///     let mut n6 = n3.child_mut(0).unwrap();
    ///     n6.push(9);
    ///
    ///     let mut n7 = n6.parent_mut().unwrap().child_mut(1).unwrap();
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

    #[inline(always)]
    pub(crate) fn col_mut(&mut self) -> &mut SelfRefCol<V, M, P> {
        self.col
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
    /// // build the following tree using child_mut and parent_mut:
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
    /// let mut a = root.child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = a.parent_mut().unwrap().child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// let mut g = b.child_mut(1).unwrap();
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

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11
    let mut tree = DynTree::<_>::new(1);

    let mut root = tree.root_mut().unwrap();

    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree).push(8);

    let mut n3 = id3.node_mut(&mut tree);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree).push(9);
    id7.node_mut(&mut tree).extend([10, 11]);

    // validate the tree

    let root = tree.root().unwrap();

    let bfs: Vec<_> = root.bfs().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    let dfs: Vec<_> = root.dfs().copied().collect();
    assert_eq!(dfs, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
}

pub trait PushOutput<V: TreeVariant> {
    fn new(memory_state: MemoryState, node_ptr: &NodePtr<V>) -> Self;
}

impl<V: TreeVariant> PushOutput<V> for () {
    fn new(memory_state: MemoryState, node_ptr: &NodePtr<V>) -> Self {
        ()
    }
}

impl<V: TreeVariant> PushOutput<V> for NodeIdx<V> {
    fn new(memory_state: MemoryState, node_ptr: &NodePtr<V>) -> Self {
        Self::new(memory_state, node_ptr)
    }
}
