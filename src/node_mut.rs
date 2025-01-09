use crate::{
    helpers::{Col, N},
    iter::ChildrenMutIter,
    memory::{Auto, MemoryPolicy},
    node_ref::NodeRefCore,
    pinned_storage::{PinnedStorage, SplitRecursive},
    traversal::{
        enumerations::Val, over_mut::OverItemMut, post_order::iter_ptr::PostOrderIterPtr, OverMut,
    },
    tree_variant::RefsChildren,
    NodeIdx, TreeVariant,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_selfref_col::{NodePtr, Refs};

/// A marker trait determining the mutation flexibility of a mutable node.
pub trait NodeMutOrientation {}

/// Allows mutation of only the node itself and its descendants.
///
/// This is a safety requirement for methods such as [`children_mut`]:
///
/// * `children_mut` returns mutable children; however, with `NodeMutDown` orientation.
/// * This prevents us from having more than once mutable reference to the same node.
///
/// [`children_mut`]: crate::NodeMut::children_mut
pub struct NodeMutDown {}
impl NodeMutOrientation for NodeMutDown {}

/// Allows mutation of the node itself, its descendants and ancestors;
/// i.e., does limit.
pub struct NodeMutUpAndDown {}
impl NodeMutOrientation for NodeMutUpAndDown {}

/// A node of the tree, which in turn is a tree.
pub struct NodeMut<'a, V, M = Auto, P = SplitRecursive, O = NodeMutUpAndDown>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    O: NodeMutOrientation,
{
    col: &'a mut Col<V, M, P>,
    node_ptr: NodePtr<V>,
    phantom: PhantomData<O>,
}

impl<'a, V, M, P, MO> NodeRefCore<'a, V, M, P> for NodeMut<'a, V, M, P, MO>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    #[inline(always)]
    fn col(&self) -> &Col<V, M, P> {
        self.col
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.node_ptr
    }
}

impl<'a, V, M, P, MO> NodeMut<'a, V, M, P, MO>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    /// Returns a mutable reference to data of this node.
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
            self.push(x);
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
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P, MO>
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

    // shrink

    /// Removes this node and all of its descendants from the tree; and returns the
    /// data of this node.
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
    /// // remove n4 downwards (removes 4 and 8)
    ///
    /// let data = id4.node_mut(&mut tree).remove();
    /// assert_eq!(data, 4);
    /// assert_eq!(tree.len(), 9);
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(values, [1, 2, 3, 5, 6, 7, 9, 10, 11]);
    ///
    /// // remove n3 downwards (3, 6, 7, 9, 10, 11)
    ///
    /// let data = id3.node_mut(&mut tree).remove();
    /// assert_eq!(data, 3);
    /// assert_eq!(tree.len(), 3);
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(values, [1, 2, 5]);
    ///
    /// // remove the root: clear the entire (remaining) tree
    ///
    /// let data = tree.root_mut().unwrap().remove();
    /// assert_eq!(data, 1);
    /// assert_eq!(tree.len(), 0);
    /// assert_eq!(tree.root(), None);
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn remove(self) -> V::Item {
        // TODO: we have the option to choose any traversal here; they are all safe
        // with SelfRefCol. We can pick the fastest one after benchmarks.

        // # SAFETY: We use this shared reference to iterate over the pointers of the
        // descendent nodes. Using a mut reference to the collection, we will close
        // each of the descendent nodes that we visit. Closing a node corresponds to
        // taking its data out and emptying all of its previous and next links.
        // Close operation is lazy and does not invalidate the pointers that we the
        // shared reference to create.
        let iter = PostOrderIterPtr::<_, Val>::from((Default::default(), self.node_ptr.clone()));
        for ptr in iter {
            if ptr != self.node_ptr {
                self.col.close(&ptr);
            }
        }

        let node = unsafe { &mut *self.node_ptr.ptr_mut() };
        if let Some(parent) = node.prev_mut().get() {
            let parent = unsafe { &mut *parent.ptr_mut() };
            let sibling_idx = parent.next_mut().remove(self.node_ptr.ptr() as usize);
            debug_assert!(sibling_idx.is_some());
        }

        let root_ptr = self.col.ends().get().expect("tree is not empty");
        if root_ptr == &self.node_ptr {
            self.col.ends_mut().clear();
        }

        // # SAFETY: On the other hand, close_and_reclaim might trigger a reclaim
        // operation which moves around the nodes, invalidating other pointers;
        // however, only after 'self.node_ptr' is also closed.
        self.col.close_and_reclaim(&self.node_ptr)
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
    /// The example below demonstrates one way to build a tree using `into_parent_mut` and `into_child_mut` methods.
    /// In this approach, we start from the mutable root node.
    /// Then, we convert one mutable node to another, always having only one mutable node.
    ///
    /// See [`grow`], [`grow_iter`] and [`grow_vec`] methods to see an alternative tree building approach which makes
    /// use of node indices.
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`grow_iter`]: crate::NodeMut::grow_iter
    /// [`grow_vec`]: crate::NodeMut::grow_vec
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //        r
    /// //       ╱ ╲
    /// //      ╱   ╲
    /// //     ╱     ╲
    /// //    a       b
    /// //  ╱ | ╲    ╱ ╲
    /// // c  d  e  f   g
    /// //            ╱ | ╲
    /// //           h  i  j
    ///
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.extend(['a', 'b']);
    ///
    /// let mut a = root.into_child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = a.into_parent_mut().unwrap().into_child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// let mut g = b.into_child_mut(1).unwrap();
    /// g.extend(['h', 'i', 'j']);
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, ['r', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, ['r', 'a', 'c', 'd', 'e', 'b', 'f', 'g', 'h', 'i', 'j']);
    /// ```
    pub fn into_child_mut(self, child_index: usize) -> Option<NodeMut<'a, V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }

    /// Creates an iterator over mutable nodes of children of this node.
    ///
    /// # Safety
    ///
    /// Mutable tree nodes; i.e. `NodeMut`, has two orientation for mutations:
    ///
    /// * [`NodeMutUpAndDown`]: This is the default orientation which allows to mutate both ancestors
    ///   and descendants of the node.
    /// * [`NodeMutDown`]: This orientation allows only to mutate self and descendants of the node.
    ///   For instance, a mutable node with this orientation does not implement [`parent_mut`] or
    ///   [`into_parent_mut`] methods.
    ///
    /// The `children_mut` iterator yields mutable nodes with `NodeMutDown` orientation.
    /// Therefore, mutating children of a node is safe, since the node itself or its ancestors cannot be mutated
    /// during the iteration.
    ///
    /// [`parent_mut`]: Self::parent_mut
    /// [`into_parent_mut`]: Self::into_parent_mut
    ///
    /// # Examples
    ///
    /// In the following example, we first build the tree; however:
    ///
    /// * we do not add nodes 8 & 9; and
    /// * we add nodes 11 & 12 with initial values of 711 & 712.
    ///
    /// Later using `children_mut` of node 2, we grow the tree by adding nodes 8 & 9.
    /// This demonstrates that we can safely mutate the structure of the tree.
    ///
    /// Then, using `children_mut` of node 7, we update values of its children.
    /// This demonstrates the mutation of node data.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //       1
    /// //      ╱ ╲
    /// //     ╱   ╲
    /// //    ╱     ╲
    /// //   2       3
    /// //  ╱ ╲     ╱  ╲
    /// // 4   5   6    7
    /// // |   |   |   ╱ ╲
    /// // *8  *9 10 *11 *12
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    ///
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// n2.extend([4, 5]);
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// id6.node_mut(&mut tree).push(10);
    /// id7.node_mut(&mut tree).extend([711, 712]);
    ///
    /// // push nodes 8 and 9 using children_mut of node 2
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// for mut child in n2.children_mut() {
    ///     let child_val = *child.data(); // 4 & 5
    ///     child.push(child_val + 4); // 8 & 9
    /// }
    ///
    /// // update values using children_mut of node 7
    ///
    /// let mut n7 = id7.node_mut(&mut tree);
    /// for mut child in n7.children_mut() {
    ///     *child.data_mut() -= 700;
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 8, 5, 9, 3, 6, 10, 7, 11, 12]);
    /// ```
    pub fn children_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = NodeMut<'_, V, M, P, NodeMutDown>>
           + DoubleEndedIterator
           + use<'_, 'a, V, M, P, MO> {
        ChildrenMutIter::new(self.col, self.node_ptr.ptr())
    }

    // dfs

    /// Creates a mutable depth first search iterator over the data of the nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` which yields [`data_mut`] of each traversed node.
    ///
    /// See also [`dfs_mut_over`] for variants yielding different values for each traversed node.
    ///
    /// [`dfs_mut_over`]: crate::NodeMut::dfs_mut_over
    /// [`data_mut`]: crate::NodeMut::data_mut
    ///
    /// # Allocation
    ///
    /// Note that depth first search requires a stack (alloc::vec::Vec) to be allocated.
    /// Each time this method is called, a stack is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`Dfs`] traverser, which can be created using [`Traversal::dfs`] method.
    /// By this, we would allocate the stack only once and re-use it to create many iterators.
    ///
    /// [`Dfs`]: crate::traversal::depth_first::Dfs
    /// [`Traversal::dfs`]: crate::Traversal::dfs
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
    /// let mut n3 = id3.node_mut(&mut tree);
    /// for x in n3.dfs_mut() {
    ///     *x /= 10;
    /// }
    /// let values: Vec<_> = id3.node(&tree).dfs().copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    ///
    /// let mut n6 = id6.node_mut(&mut tree);
    /// for x in n6.dfs_mut() {
    ///     *x *= 100;
    /// }
    /// let values: Vec<_> = id6.node(&tree).dfs().copied().collect();
    /// assert_eq!(values, [600, 900]);
    ///
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(values, [10, 20, 40, 80, 50, 3, 600, 900, 7, 10, 11]);
    /// ```
    pub fn dfs_mut(&mut self) -> impl Iterator<Item = &mut V::Item> {
        use crate::traversal::depth_first::{iter_mut::DfsIterMut, iter_ptr::DfsIterPtr};
        let root = self.node_ptr().clone();
        let iter = DfsIterPtr::<_, Val>::from((Default::default(), root));
        unsafe { DfsIterMut::<'_, _, M, P, _, _, _>::from((self.col, iter)) }
    }

    /// Creates a mutable depth first search iterator over different values of nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic [`OverMut`] type parameter `O`.
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// # Allocation
    ///
    /// Note that depth first search requires a stack (alloc::vec::Vec) to be allocated.
    /// Each time this method is called, a stack is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`Dfs`] traverser, which can be created using [`Traversal::dfs`] method.
    /// By this, we would allocate the stack only once and re-use it to create many iterators.
    ///
    /// [`Dfs`]: crate::traversal::depth_first::Dfs
    /// [`Traversal::dfs`]: crate::Traversal::dfs
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::traversal::*;
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
    ///
    ///     let mut tree = DynTree::<i32>::new(1);
    ///
    ///     let mut root = tree.root_mut().unwrap();
    ///     let [id2, id3] = root.grow([2, 3]);
    ///
    ///     let mut n2 = id2.node_mut(&mut tree);
    ///     let [id4, _] = n2.grow([4, 5]);
    ///
    ///     id4.node_mut(&mut tree).push(8);
    ///
    ///     let mut n3 = id3.node_mut(&mut tree);
    ///     let [id6, id7] = n3.grow([6, 7]);
    ///
    ///     id6.node_mut(&mut tree).push(9);
    ///     id7.node_mut(&mut tree).extend([10, 11]);
    ///
    ///     tree
    /// }
    ///
    /// // dfs over data_mut
    ///
    /// let mut tree = init_tree();
    ///
    /// let mut root = tree.root_mut().unwrap();
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
    /// let mut root = tree.root_mut().unwrap();
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
    /// let mut root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, data) in root.dfs_mut_over::<OverDepthSiblingIdxData>() {
    ///     *data += depth as i32 * 100 + sibling_idx as i32 * 10000;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [1, 102, 204, 308, 10205, 10103, 206, 309, 10207, 310, 10311]
    /// );
    /// ```
    pub fn dfs_mut_over<O: OverMut>(
        &mut self,
    ) -> impl Iterator<Item = OverItemMut<'_, V, O, M, P>> {
        use crate::traversal::depth_first::{iter_mut::DfsIterMut, iter_ptr::DfsIterPtr};
        let root = self.node_ptr().clone();
        let iter = DfsIterPtr::<_, O::Enumeration>::from((Default::default(), root));
        unsafe { DfsIterMut::from((self.col, iter)) }
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
    /// Note that breadth first search requires a queue (alloc::collections::VecDeque) to be allocated.
    /// Each time this method is called, a queue is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`Bfs`] traverser, which can be created using [`Traversal::bfs`] method.
    /// By this, we would allocate the queue only once and re-use it to create many iterators.
    ///
    /// [`Bfs`]: crate::traversal::breadth_first::Bfs
    /// [`Traversal::bfs`]: crate::Traversal::bfs
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
    /// let mut n3 = id3.node_mut(&mut tree);
    /// for x in n3.bfs_mut() {
    ///     *x /= 10;
    /// }
    /// let values: Vec<_> = id3.node(&tree).bfs().copied().collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    ///
    /// let mut n6 = id6.node_mut(&mut tree);
    /// for x in n6.bfs_mut() {
    ///     *x *= 100;
    /// }
    /// let values: Vec<_> = id6.node(&tree).bfs().copied().collect();
    /// assert_eq!(values, [600, 900]);
    ///
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(values, [10, 20, 3, 40, 50, 600, 7, 80, 900, 10, 11]);
    /// ```
    pub fn bfs_mut(&mut self) -> impl Iterator<Item = &mut V::Item> {
        use crate::traversal::breadth_first::{iter_mut::BfsIterMut, iter_ptr::BfsIterPtr};
        let root = self.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val>::from((Default::default(), root));
        unsafe { BfsIterMut::<'_, _, M, P, _, _, _>::from((self.col, iter)) }
    }

    /// Creates a mutable breadth first search iterator over different values of nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic [`OverMut`] type parameter `O`.
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// # Allocation
    ///
    /// Note that breadth first search requires a queue (alloc::collections::VecDeque) to be allocated.
    /// Each time this method is called, a queue is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`Bfs`] traverser, which can be created using [`Traversal::bfs`] method.
    /// By this, we would allocate the queue only once and re-use it to create many iterators.
    ///
    /// [`Bfs`]: crate::traversal::breadth_first::Bfs
    /// [`Traversal::bfs`]: crate::Traversal::bfs
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::traversal::*;
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
    ///
    ///     let mut tree = DynTree::<i32>::new(1);
    ///
    ///     let mut root = tree.root_mut().unwrap();
    ///     let [id2, id3] = root.grow([2, 3]);
    ///
    ///     let mut n2 = id2.node_mut(&mut tree);
    ///     let [id4, _] = n2.grow([4, 5]);
    ///
    ///     id4.node_mut(&mut tree).push(8);
    ///
    ///     let mut n3 = id3.node_mut(&mut tree);
    ///     let [id6, id7] = n3.grow([6, 7]);
    ///
    ///     id6.node_mut(&mut tree).push(9);
    ///     id7.node_mut(&mut tree).extend([10, 11]);
    ///
    ///     tree
    /// }
    ///
    /// // bfs over data_mut
    ///
    /// let mut tree = init_tree();
    ///
    /// let mut root = tree.root_mut().unwrap();
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
    /// let mut root = tree.root_mut().unwrap();
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
    /// let mut root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, data) in root.bfs_mut_over::<OverDepthSiblingIdxData>() {
    ///     *data += depth as i32 * 100 + sibling_idx as i32 * 10000;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [1, 102, 10103, 204, 10205, 206, 10207, 308, 309, 310, 10311]
    /// );
    /// ```
    pub fn bfs_mut_over<O: OverMut>(
        &mut self,
    ) -> impl Iterator<Item = OverItemMut<'_, V, O, M, P>> {
        use crate::traversal::breadth_first::{iter_mut::BfsIterMut, iter_ptr::BfsIterPtr};
        let root = self.node_ptr().clone();
        let iter = BfsIterPtr::<_, O::Enumeration>::from((Default::default(), root));
        unsafe { BfsIterMut::from((self.col, iter)) }
    }

    // post-order

    /// Creates a mutable iterator for post-order traversal
    /// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
    ///
    /// An important property of post-order traversal is that nodes are yield after their all descendants are
    /// yield; and hence, the root (this) node will be yield at last.
    /// Among other reasons, this makes post-order traversal very useful for pruning or removing nodes from trees.
    ///
    /// Return value is an `Iterator` which yields [`data_mut`] of each traversed node.
    ///
    /// See also [`post_order_mut_over`] for variants yielding different values for each traversed node.
    ///
    /// [`data_mut`]: crate::NodeMut::data_mut
    /// [`post_order_mut_over`]: crate::NodeMut::post_order_mut_over
    ///
    /// # Allocation
    ///
    /// Note that post order traversal requires a vector (alloc::vec::Vec) to be allocated, with a length equal to
    /// the max depth of the traversed nodes.
    /// Each time this method is called, a vector is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`PostOrder`] traverser, which can be created using [`Traversal::post_order`] method.
    /// By this, we would allocate the vector only once and re-use it to create many iterators.
    ///
    /// [`PostOrder`]: crate::traversal::post_order::PostOrder
    /// [`Traversal::post_order`]: crate::Traversal::post_order
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
    /// // traversal from the root
    ///
    /// for x in tree.root_mut().unwrap().post_order_mut() {
    ///     *x *= 10;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().post_order().copied().collect();
    /// assert_eq!(values, [80, 40, 50, 20, 90, 60, 100, 110, 70, 30, 10]);
    ///
    /// // traversal from any node
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// for x in n3.bfs_mut() {
    ///     *x /= 10;
    /// }
    /// let values: Vec<_> = id3.node(&tree).post_order().copied().collect();
    /// assert_eq!(values, [9, 6, 10, 11, 7, 3]);
    ///
    /// let mut n6 = id6.node_mut(&mut tree);
    /// for x in n6.bfs_mut() {
    ///     *x *= 100;
    /// }
    /// let values: Vec<_> = id6.node(&tree).post_order().copied().collect();
    /// assert_eq!(values, [900, 600]);
    ///
    /// let values: Vec<_> = tree.root().unwrap().post_order().copied().collect();
    /// assert_eq!(values, [80, 40, 50, 20, 900, 600, 10, 11, 7, 3, 10]);
    /// ```
    pub fn post_order_mut(&mut self) -> impl Iterator<Item = &mut V::Item> {
        use crate::traversal::post_order::{
            iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr,
        };
        let root = self.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val>::from((Default::default(), root));
        unsafe { PostOrderIterMut::<'_, _, M, P, _, _, _>::from((self.col, iter)) }
    }

    /// Creates a mutable iterator for post-order traversal
    /// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
    ///
    /// An important property of post-order traversal is that nodes are yield after their all descendants are
    /// yield; and hence, the root (this) node will be yield at last.
    /// Among other reasons, this makes post-order traversal very useful for pruning or removing nodes from trees.
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic [`OverMut`] type parameter `O`.
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// # Allocation
    ///
    /// Note that post order traversal requires a vector (alloc::vec::Vec) to be allocated, with a length equal to
    /// the max depth of the traversed nodes.
    /// Each time this method is called, a vector is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`PostOrder`] traverser, which can be created using [`Traversal::post_order`] method.
    /// By this, we would allocate the vector only once and re-use it to create many iterators.
    ///
    /// [`PostOrder`]: crate::traversal::post_order::PostOrder
    /// [`Traversal::post_order`]: crate::Traversal::post_order
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::traversal::*;
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
    ///
    ///     let mut tree = DynTree::<i32>::new(1);
    ///
    ///     let mut root = tree.root_mut().unwrap();
    ///     let [id2, id3] = root.grow([2, 3]);
    ///
    ///     let mut n2 = id2.node_mut(&mut tree);
    ///     let [id4, _] = n2.grow([4, 5]);
    ///
    ///     id4.node_mut(&mut tree).push(8);
    ///
    ///     let mut n3 = id3.node_mut(&mut tree);
    ///     let [id6, id7] = n3.grow([6, 7]);
    ///
    ///     id6.node_mut(&mut tree).push(9);
    ///     id7.node_mut(&mut tree).extend([10, 11]);
    ///
    ///     tree
    /// }
    ///
    /// // post-order over data_mut
    ///
    /// let mut tree = init_tree();
    ///
    /// let mut root = tree.root_mut().unwrap();
    ///
    /// // equivalent to `root.post_order_mut()`
    /// for data in root.post_order_mut_over::<OverData>() {
    ///     *data += 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().post_order().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [108, 104, 105, 102, 109, 106, 110, 111, 107, 103, 101]
    /// );
    ///
    /// // post-order over (depth, data_mut)
    ///
    /// let mut tree = init_tree();
    ///
    /// let mut root = tree.root_mut().unwrap();
    ///
    /// for (depth, data) in root.post_order_mut_over::<OverDepthData>() {
    ///     *data += depth as i32 * 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().post_order().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [308, 204, 205, 102, 309, 206, 310, 311, 207, 103, 1]
    /// );
    ///
    /// // post-order over (depth, sibling index, data_mut)
    ///
    /// let mut tree = init_tree();
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// for (depth, sibling_idx, data) in root.post_order_mut_over::<OverDepthSiblingIdxData>() {
    ///     *data += depth as i32 * 100 + sibling_idx as i32 * 10000;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().post_order().copied().collect();
    /// assert_eq!(
    ///     values,
    ///     [308, 204, 10205, 102, 309, 206, 310, 10311, 10207, 10103, 1]
    /// );
    /// ```
    pub fn post_order_mut_over<O: OverMut>(
        &mut self,
    ) -> impl Iterator<Item = OverItemMut<'_, V, O, M, P>> {
        use crate::traversal::post_order::{
            iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr,
        };
        let root = self.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, O::Enumeration>::from((Default::default(), root));
        unsafe { PostOrderIterMut::from((self.col, iter)) }
    }

    /// Similar to [`remove`], this method removes this node and all of its descendants from the tree.
    ///
    /// However, they differ in the returned value:
    ///
    /// * `remove` returns only the data of this node and ignores the data of the descendants,
    /// * `remove_post_order` returns an iterator which returns the data of all descendants of
    ///   this node in the order of the **post-order** traversal
    ///   ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
    ///
    /// [`remove`]: Self::remove
    ///
    /// # Allocation
    ///
    /// Note that post order traversal requires a vector (alloc::vec::Vec) to be allocated, with a length equal to
    /// the max depth of the traversed nodes.
    /// Each time this method is called, a vector is allocated, used and dropped.
    ///
    /// For situations where we repeatedly traverse over the tree and the allocation might be considered expensive,
    /// it is recommended to use the [`PostOrder`] traverser, which can be created using [`Traversal::post_order`] method.
    /// By this, we would allocate the vector only once and re-use it to create many iterators.
    ///
    /// [`PostOrder`]: crate::traversal::post_order::PostOrder
    /// [`Traversal::post_order`]: crate::Traversal::post_order
    ///
    /// # Example
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
    /// // remove node 3 and its descendants
    /// // collect the removed values into a vector in the traversal's order
    /// let n3 = id3.node_mut(&mut tree);
    /// let removed_values: Vec<_> = n3.remove_post_order().collect();
    /// assert_eq!(removed_values, [9, 6, 10, 11, 7, 3]);
    ///
    /// let remaining_values: Vec<_> = tree.root().unwrap().post_order().copied().collect();
    /// assert_eq!(remaining_values, [8, 4, 5, 2, 1]);
    ///
    /// // let's remove root and its descendants (empty the tree)
    /// // and collect remaining nodes in the traversal's order
    ///
    /// let root = tree.root_mut().unwrap();
    /// let removed_values: Vec<_> = root.remove_post_order().collect();
    /// assert_eq!(removed_values, [8, 4, 5, 2, 1]);
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.root(), None);
    /// ```
    pub fn remove_post_order(self) -> impl Iterator<Item = V::Item> + use<'a, V, M, P, MO> {
        let ptr = self.node_ptr.clone();
        use crate::traversal::post_order::{
            into_iter::PostOrderIterInto, iter_ptr::PostOrderIterPtr,
        };
        let root = self.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val>::from((Default::default(), root));
        unsafe { PostOrderIterInto::<V, M, P, Val, _>::from((self.col, iter, ptr)) }
    }

    // helpers

    pub(crate) fn new(col: &'a mut Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
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

    pub(crate) fn into_inner(self) -> (&'a mut Col<V, M, P>, NodePtr<V>) {
        (self.col, self.node_ptr)
    }
}

impl<'a, V, M, P> NodeMut<'a, V, M, P, NodeMutUpAndDown>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    // traversal

    /// Returns the mutable node of this node's parent,
    /// returns None if this is the root node.
    ///
    /// See also [`into_parent_mut`] for consuming traversal.
    ///
    /// [`into_parent_mut`]: crate::NodeMut::into_parent_mut
    ///
    /// # Examples
    ///
    /// The example below demonstrates one way to build a tree using `into_parent_mut` and `into_child_mut` methods.
    /// In this approach, we start from the mutable root node.
    /// Then, we convert one mutable node to another, always having only one mutable node.
    ///
    /// See [`grow`], [`grow_iter`] and [`grow_vec`] methods to see an alternative tree building approach which makes
    /// use of node indices.
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`grow_iter`]: crate::NodeMut::grow_iter
    /// [`grow_vec`]: crate::NodeMut::grow_vec
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //        x
    /// //       ╱ ╲
    /// //      ╱   ╲
    /// //     ╱     ╲
    /// //    a       b
    /// //  ╱ | ╲    ╱ ╲
    /// // c  d  e  f   g
    ///
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// let [id_a, id_b] = root.grow(['a', 'b']);
    ///
    /// let mut a = id_a.node_mut(&mut tree);
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = id_b.node_mut(&mut tree);
    /// let [_, id_g] = b.grow(['f', 'g']);
    ///
    /// let mut g = id_g.node_mut(&mut tree);
    /// let mut b = g.parent_mut().unwrap();
    /// let mut root = b.parent_mut().unwrap();
    ///
    /// *root.data_mut() = 'x';
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, ['x', 'a', 'b', 'c', 'd', 'e', 'f', 'g']);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, ['x', 'a', 'c', 'd', 'e', 'b', 'f', 'g']);
    /// ```
    pub fn parent_mut(&mut self) -> Option<NodeMut<'_, V, M, P>> {
        self.node()
            .prev()
            .get()
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }

    /// Consumes this mutable node and returns the mutable node of its parent,
    /// returns None if this is the root node.
    ///
    /// See also [`parent_mut`] for non-consuming access.
    ///
    /// [`parent_mut`]: crate::NodeMut::parent_mut
    ///
    /// # Examples
    ///
    /// The example below demonstrates one way to build a tree using `into_parent_mut` and `into_child_mut` methods.
    /// In this approach, we start from the mutable root node.
    /// Then, we convert one mutable node to another, always having only one mutable node.
    ///
    /// See [`grow`], [`grow_iter`] and [`grow_vec`] methods to see an alternative tree building approach which makes
    /// use of node indices.
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`grow_iter`]: crate::NodeMut::grow_iter
    /// [`grow_vec`]: crate::NodeMut::grow_vec
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //        r
    /// //       ╱ ╲
    /// //      ╱   ╲
    /// //     ╱     ╲
    /// //    a       b
    /// //  ╱ | ╲    ╱ ╲
    /// // c  d  e  f   g
    /// //            ╱ | ╲
    /// //           h  i  j
    ///
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.extend(['a', 'b']);
    ///
    /// let mut a = root.into_child_mut(0).unwrap();
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = a.into_parent_mut().unwrap().into_child_mut(1).unwrap();
    /// b.extend(['f', 'g']);
    ///
    /// let mut g = b.into_child_mut(1).unwrap();
    /// g.extend(['h', 'i', 'j']);
    ///
    /// // validate the tree
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let bfs: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(bfs, ['r', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
    ///
    /// let dfs: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(dfs, ['r', 'a', 'c', 'd', 'e', 'b', 'f', 'g', 'h', 'i', 'j']);
    /// ```
    pub fn into_parent_mut(self) -> Option<NodeMut<'a, V, M, P>> {
        self.node()
            .prev()
            .get()
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }
}
