use crate::{
    helpers::{Col, N},
    iter::AncestorsIterPtr,
    memory::MemoryPolicy,
    pinned_storage::PinnedStorage,
    traversal::{
        enumeration::Enumeration, enumerations::Val, over::OverItem, traverser_core::TraverserCore,
        Over, OverData,
    },
    tree_variant::RefsChildren,
    Node, NodeIdx, Traverser, TreeVariant,
};
use orx_selfref_col::{NodePtr, Refs};

pub trait NodeRefCore<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn col(&self) -> &Col<V, M, P>;

    fn node_ptr(&self) -> &NodePtr<V>;

    #[inline(always)]
    fn node(&self) -> &N<V> {
        unsafe { &*self.node_ptr().ptr() }
    }
}

impl<'a, V, M, P, X> NodeRef<'a, V, M, P> for X
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    X: NodeRefCore<'a, V, M, P>,
{
}

/// Reference to a tree node.
pub trait NodeRef<'a, V, M, P>: NodeRefCore<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    /// Returns the node index of this node.
    ///
    /// TODO: examples
    fn idx(&self) -> NodeIdx<V> {
        NodeIdx(orx_selfref_col::NodeIdx::new(
            self.col().memory_state(),
            self.node_ptr(),
        ))
    }

    /// Returns true if this is the root node; equivalently, if its [`parent`] is none.
    ///
    /// [`parent`]: NodeRef::parent
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = BinaryTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut();
    /// assert!(root.is_root());
    ///
    /// root.extend(['a', 'b']);
    /// for node in root.children() {
    ///     assert!(!node.is_root());
    /// }
    /// ```
    #[inline(always)]
    fn is_root(&self) -> bool {
        self.node().prev().get().is_none()
    }

    /// Returns true if this is a leaf node; equivalently, if [`num_children`] is zero.
    ///
    /// [`num_children`]: NodeRef::num_children
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //  ╱ ╲   ╱
    /// // 4   5 6
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    /// assert_eq!(tree.get_root().unwrap().is_leaf(), true); // both root & leaf
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, id5] = n2.grow([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6] = n3.grow([6]);
    ///
    /// // walk over any subtree rooted at a selected node
    /// // with different traversals
    ///
    /// assert_eq!(tree.get_root().unwrap().is_leaf(), false);
    /// assert_eq!(tree.node(&id2).is_leaf(), false);
    /// assert_eq!(tree.node(&id3).is_leaf(), false);
    ///
    /// assert_eq!(tree.node(&id4).is_leaf(), true);
    /// assert_eq!(tree.node(&id5).is_leaf(), true);
    /// assert_eq!(tree.node(&id6).is_leaf(), true);
    /// ```
    #[inline(always)]
    fn is_leaf(&self) -> bool {
        self.num_children() == 0
    }

    /// Returns a reference to the data of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut();
    /// assert_eq!(root.data(), &0);
    ///
    /// let [id_a] = root.grow([1]);
    /// let a = tree.node(&id_a);
    /// assert_eq!(a.data(), &1);
    /// ```
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    fn data<'b>(&'b self) -> &'b V::Item
    where
        'a: 'b,
    {
        self.node()
            .data()
            .expect("node holding a tree reference must be active")
    }

    /// Returns the number of child nodes of this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut();
    /// assert_eq!(root.num_children(), 0);
    ///
    /// let [id_a, id_b] = root.grow([1, 2]);
    /// assert_eq!(root.num_children(), 2);
    ///
    /// let mut node = tree.node_mut(&id_a);
    /// node.push(3);
    /// node.extend([4, 5, 6]);
    /// assert_eq!(node.num_children(), 4);
    ///
    /// assert_eq!(tree.node(&id_b).num_children(), 0);
    /// ```
    #[inline(always)]
    fn num_children(&self) -> usize {
        self.node().next().num_children()
    }

    /// Returns an exact-sized iterator of children nodes of this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the tree:
    /// // r
    /// // |-- a
    /// //     |-- c, d, e
    /// // |-- b
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut();
    /// let [id_a] = root.grow(['a']);
    /// root.push('b');
    ///
    /// let mut a = tree.node_mut(&id_a);
    /// a.extend(['c', 'd', 'e']);
    ///
    /// // iterate over children of nodes
    ///
    /// let root = tree.root();
    /// let depth1: Vec<_> = root.children().map(|x| *x.data()).collect();
    /// assert_eq!(depth1, ['a', 'b']);
    ///
    /// let b = root.children().nth(0).unwrap();
    /// let depth2: Vec<_> = b.children().map(|x| *x.data()).collect();
    /// assert_eq!(depth2, ['c', 'd', 'e']);
    ///
    /// // depth first - two levels deep
    /// let mut data = vec![];
    /// for node in root.children() {
    ///     data.push(*node.data());
    ///
    ///     for child in node.children() {
    ///         data.push(*child.data());
    ///     }
    /// }
    ///
    /// assert_eq!(data, ['a', 'c', 'd', 'e', 'b']);
    /// ```
    fn children(&'a self) -> impl ExactSizeIterator<Item = Node<'a, V, M, P>> {
        self.node()
            .next()
            .children_ptr()
            .map(|ptr| Node::new(self.col(), ptr.clone()))
    }

    /// Returns the `child-index`-th child of the node; returns None if out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the tree:
    /// // r
    /// // |-- a
    /// //     |-- c, d, e
    /// // |-- b
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut();
    /// let [id_a] = root.grow(['a']);
    /// root.push('b');
    ///
    /// let mut a = tree.node_mut(&id_a);
    /// a.extend(['c', 'd', 'e']);
    ///
    /// // use child to access lower level nodes
    ///
    /// let root = tree.root();
    ///
    /// let a = root.child(0).unwrap();
    /// assert_eq!(a.data(), &'a');
    /// assert_eq!(a.num_children(), 3);
    ///
    /// assert_eq!(a.child(1).unwrap().data(), &'d');
    /// assert_eq!(a.child(3), None);
    /// ```
    fn child(&self, child_index: usize) -> Option<Node<V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
            .map(|ptr| Node::new(self.col(), ptr.clone()))
    }

    /// Returns the parent of this node; returns None if this is the root node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = BinaryTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut();
    /// assert_eq!(root.parent(), None);
    ///
    /// root.extend(['a', 'b']);
    /// for node in root.children() {
    ///     assert_eq!(node.parent().unwrap(), root);
    /// }
    /// ```
    fn parent(&self) -> Option<Node<V, M, P>> {
        self.node()
            .prev()
            .get()
            .map(|ptr| Node::new(self.col(), ptr.clone()))
    }

    /// Returns the position of this node in the children collection of its parent;
    /// returns 0 if this is the root node (root has no other siblings).
    ///
    /// **O(S)** where S is the number of siblings; i.e.,
    /// requires linear search over the children of the parent of this node.
    /// Therefore, S depends on the tree size. It bounded by 2 in a [`BinaryTree`],
    /// by 4 in a [`DaryTree`] with D=4. In a [`DynTree`] the children list can grow
    /// arbitrarily, therefore, it is not bounded.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
    /// [`DynTree`]: crate::DynTree
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      r
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   a     b
    /// //  ╱|╲   ╱ ╲
    /// // c d e f   g
    /// //          ╱|╲
    /// //         h i j
    ///
    /// let mut tree = DynTree::<char>::new('r');
    ///
    /// let mut root = tree.root_mut();
    /// let [id_a, id_b] = root.grow(['a', 'b']);
    ///
    /// let mut a = tree.node_mut(&id_a);
    /// a.extend(['c', 'd', 'e']);
    ///
    /// let mut b = tree.node_mut(&id_b);
    /// let [_, id_g] = b.grow(['f', 'g']);
    ///
    /// let mut g = tree.node_mut(&id_g);
    /// let [id_h, id_i, id_j] = g.grow(['h', 'i', 'j']);
    ///
    /// // check sibling positions
    ///
    /// let root = tree.root();
    /// assert_eq!(root.sibling_idx(), 0);
    ///
    /// for (i, node) in root.children().enumerate() {
    ///     assert_eq!(node.sibling_idx(), i);
    /// }
    ///
    /// assert_eq!(tree.node(&id_h).sibling_idx(), 0);
    /// assert_eq!(tree.node(&id_i).sibling_idx(), 1);
    /// assert_eq!(tree.node(&id_j).sibling_idx(), 2);
    /// ```
    fn sibling_idx(&self) -> usize {
        let parent = self.node().prev().get().map(|ptr| unsafe { ptr.node() });
        parent
            .map(|parent| {
                let ptr = self.node_ptr();
                let mut children = parent.next().children_ptr();
                children.position(|x| x == ptr).expect("this node exists")
            })
            .unwrap_or(0)
    }

    /// Returns the depth of this node with respect to the root of the tree which has a
    /// depth of 0.
    ///
    /// **O(D)** requires linear time in maximum depth of the tree.
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
    /// // |
    /// // 8
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, id5] = n2.grow([4, 5]);
    ///
    /// let [id8] = tree.node_mut(&id4).grow([8]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// // access the leaves in different orders that is determined by traversal
    ///
    /// assert_eq!(tree.root().depth(), 0);
    ///
    /// assert_eq!(tree.node(&id2).depth(), 1);
    /// assert_eq!(tree.node(&id3).depth(), 1);
    ///
    /// assert_eq!(tree.node(&id4).depth(), 2);
    /// assert_eq!(tree.node(&id5).depth(), 2);
    /// assert_eq!(tree.node(&id6).depth(), 2);
    /// assert_eq!(tree.node(&id7).depth(), 2);
    ///
    /// assert_eq!(tree.node(&id8).depth(), 3);
    /// ```
    fn depth(&self) -> usize {
        let mut depth = 0;

        let mut current = unsafe { &*self.node_ptr().ptr() };
        while let Some(parent_ptr) = current.prev().get() {
            depth += 1;
            current = unsafe { &*parent_ptr.ptr() };
        }

        depth
    }

    /// Returns an iterator starting from this node moving upwards until the root:
    ///
    /// * yields all ancestors of this node including this node,
    /// * the first element is always this node, and
    /// * the last element is always the root node of the tree.
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// let [id10, _] = tree.node_mut(&id7).grow([10, 11]);
    ///
    /// // ancestors iterator over nodes
    /// // upwards from the node to the root
    ///
    /// let root = tree.root();
    /// let mut iter = root.ancestors();
    /// assert_eq!(iter.next().as_ref(), Some(&root));
    /// assert_eq!(iter.next(), None);
    ///
    /// let n10 = tree.node(&id10);
    /// let ancestors_data: Vec<_> = n10.ancestors().map(|x| *x.data()).collect();
    /// assert_eq!(ancestors_data, [10, 7, 3, 1]);
    ///
    /// let n4 = tree.node(&id4);
    /// let ancestors_data: Vec<_> = n4.ancestors().map(|x| *x.data()).collect();
    /// assert_eq!(ancestors_data, [4, 2, 1]);
    /// ```
    fn ancestors(&'a self) -> impl Iterator<Item = Node<'a, V, M, P>> {
        let root_ptr = self.col().ends().get().expect("Tree is non-empty").clone();
        AncestorsIterPtr::new(root_ptr, self.node_ptr().clone())
            .map(|ptr| Node::new(self.col(), ptr))
    }

    // traversal

    /// Creates an iterator that yields references to data of all nodes belonging to the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the generic [`Traverser`] parameter `T`.
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Bfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// # See also
    ///
    /// See also [`walk_mut`] and [`into_walk`] for iterators over mutable references and owned (removed) values,
    /// respectively.
    ///
    /// Note that tree traversing methods typically allocate a temporary data structure that is dropped once the
    /// iterator is dropped.
    /// In use cases where we repeatedly iterate using any of the **walk** methods over different nodes or different
    /// trees, we can avoid the allocation by creating the traverser only once and using [`walk_with`], [`walk_mut_with`]
    /// and [`into_walk_with`] methods instead.
    /// These methods additionally allow for iterating over nodes rather than data; and yielding node depths and sibling
    /// indices in addition to node data.
    ///
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
    /// [`walk_mut`]: crate::NodeMut::walk_mut
    /// [`into_walk`]: crate::NodeMut::into_walk
    /// [`walk_with`]: crate::NodeRef::walk_with
    /// [`walk_mut_with`]: crate::NodeMut::walk_mut_with
    /// [`into_walk_with`]: crate::NodeMut::into_walk_with
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // walk over any subtree rooted at a selected node
    /// // with different traversals
    ///
    /// let root = tree.root();
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// let n3 = tree.node(&id3);
    /// let dfs: Vec<_> = n3.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);
    ///
    /// let n2 = tree.node(&id2);
    /// let post_order: Vec<_> = n2.walk::<PostOrder>().copied().collect();
    /// assert_eq!(post_order, [8, 4, 5, 2]);
    /// ```
    fn walk<T>(&'a self) -> impl Iterator<Item = &'a V::Item>
    where
        T: Traverser<OverData>,
        Self: Sized,
    {
        T::iter_with_owned_storage::<V, M, P>(self)
    }

    /// Creates an iterator that traverses all nodes belonging to the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`walk`], this method does require internal allocation.
    /// Furthermore, it allows to iterate over nodes rather than data; and to attach node depths or sibling
    /// indices to the yield values.
    /// Please see the examples below.
    ///
    /// [`walk`]: crate::NodeRef::walk
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
    ///
    /// # Examples
    ///
    /// ## Examples - Repeated Iterations without Allocation
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // create the traverser 'dfs' only once, use it many times
    /// // to walk over references, mutable references or removed values
    /// // without additional allocation
    ///
    /// let mut dfs = Dfs::default();
    ///
    /// let root = tree.root();
    /// let values: Vec<_> = root.walk_with(&mut dfs).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// let mut n7 = tree.node_mut(&id7);
    /// for x in n7.walk_mut_with(&mut dfs) {
    ///     *x += 100;
    /// }
    /// let values: Vec<_> = tree.get_root().unwrap().walk_with(&mut dfs).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 107, 110, 111]);
    ///
    /// let n3 = tree.node_mut(&id3);
    /// let removed: Vec<_> = n3.into_walk_with(&mut dfs).collect();
    /// assert_eq!(removed, [3, 6, 9, 107, 110, 111]);
    ///
    /// let remaining: Vec<_> = tree.get_root().unwrap().walk_with(&mut dfs).copied().collect();
    /// assert_eq!(remaining, [1, 2, 4, 8, 5]);
    /// ```
    ///
    /// ## Examples - Yielding Different Items
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // create the traverser 'bfs' iterator
    /// // to walk over nodes rather than data
    ///
    /// let mut bfs = Bfs::default().over_nodes();
    /// // OR: Bfs::<OverNode>::new();
    ///
    /// let n7 = tree.node(&id7);
    /// let mut iter = n7.walk_with(&mut bfs);
    /// let node = iter.next().unwrap();
    /// assert_eq!(node.num_children(), 2);
    /// assert_eq!(node.child(1).map(|x| *x.data()), Some(11));
    ///
    /// // or to additionally yield depth and/or sibling-idx
    ///
    /// let mut dfs = Dfs::default().with_depth().with_sibling_idx();
    /// // OR: Dfs::<OverDepthSiblingIdxData>::new()
    ///
    /// let n3 = tree.node(&id3);
    /// let result: Vec<_> = n3
    ///     .walk_with(&mut dfs)
    ///     .map(|(depth, sibling_idx, data)| (depth, sibling_idx, *data))
    ///     .collect();
    /// assert_eq!(
    ///     result,
    ///     [
    ///         (0, 0, 3),
    ///         (1, 0, 6),
    ///         (2, 0, 9),
    ///         (1, 1, 7),
    ///         (2, 0, 10),
    ///         (2, 1, 11)
    ///     ]
    /// );
    /// ```
    fn walk_with<T, O>(
        &'a self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        O: Over,
        T: Traverser<O>,
        Self: Sized,
    {
        traverser.iter(self)
    }

    /// Returns an iterator of paths from all leaves of the subtree rooted at
    /// this node **upwards** to this node.
    ///
    /// The iterator yields one path per leaf node.
    ///
    /// The order of the leaves, and hence the corresponding paths, is determined
    /// by the generic [`Traverser`] parameter `T`.
    ///
    /// # See also
    ///
    /// * [`paths_with`]: (i) to iterate using a cached traverser to minimize allocation
    ///   for repeated traversals, or (ii) to iterate over nodes rather than only the data.
    ///
    /// [`paths_with`]: NodeRef::paths_with
    ///
    /// # Yields
    ///
    /// * `Iterator::Item` => `impl Iterator<Item = &'a V::Item>`
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // paths from all leaves to the root
    ///
    /// let root = tree.root();
    ///
    /// // sorted in the order of leaves by breadth-first:
    /// // 5, 8, 9, 10, 11
    /// let paths: Vec<_> = root
    ///     .paths::<Bfs>()
    ///     .map(|x| x.copied().collect::<Vec<_>>())
    ///     .collect();
    ///
    /// assert_eq!(
    ///     paths,
    ///     [
    ///         vec![5, 2, 1],
    ///         vec![8, 4, 2, 1],
    ///         vec![9, 6, 3, 1],
    ///         vec![10, 7, 3, 1],
    ///         vec![11, 7, 3, 1]
    ///     ]
    /// );
    ///
    /// // paths from all leaves of subtree rooted at n3
    ///
    /// let n3 = tree.node(&id3);
    ///
    /// let paths: Vec<_> = n3
    ///     .paths::<Dfs>()
    ///     .map(|x| x.copied().collect::<Vec<_>>())
    ///     .collect();
    ///
    /// assert_eq!(paths, [vec![9, 6, 3], vec![10, 7, 3], vec![11, 7, 3]]);
    /// ```
    fn paths<T>(&'a self) -> impl Iterator<Item = impl Iterator<Item = &'a V::Item>>
    where
        T: Traverser<OverData>,
    {
        let node_ptr = self.node_ptr();
        T::iter_ptr_with_owned_storage(node_ptr.clone())
            .filter(|x: &NodePtr<V>| unsafe { &*x.ptr() }.next().is_empty())
            .map(|x: NodePtr<V>| {
                let iter = AncestorsIterPtr::new(node_ptr.clone(), x);
                iter.map(|ptr| (unsafe { &*ptr.ptr() }).data().expect("active tree node"))
            })
    }

    /// Returns an iterator of paths from all leaves of the subtree rooted at
    /// this node **upwards** to this node.
    ///
    /// The iterator yields one path per leaf node.
    ///
    /// The order of the leaves, and hence the corresponding paths, is determined
    /// by explicit type of the [`Traverser`] argument `traverser`.
    ///
    /// # Yields
    ///
    /// * `Iterator::Item` => `impl Iterator<Item = &'a V::Item>`
    ///   when `T: Traverser<OverData>`
    /// * `Iterator::Item` => `impl Iterator<Item = Node<_>>`
    ///   when `T: Traverser<OverNode>`
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // create a depth first traverser and reuse it
    ///
    /// let mut dfs = Traversal.dfs(); // OverData by default
    ///
    /// // paths from leaves as data of the nodes
    ///
    /// let root = tree.root();
    /// let paths: Vec<_> = root
    ///     .paths_with(&mut dfs)
    ///     .map(|path_data| path_data.copied().collect::<Vec<_>>())
    ///     .collect();
    ///
    /// assert_eq!(
    ///     paths,
    ///     [
    ///         vec![8, 4, 2, 1],
    ///         vec![5, 2, 1],
    ///         vec![9, 6, 3, 1],
    ///         vec![10, 7, 3, 1],
    ///         vec![11, 7, 3, 1]
    ///     ]
    /// );
    ///
    /// // paths of subtree rooted at n3; as nodes rather than data.
    ///
    /// let mut dfs = dfs.over_nodes(); // transform from OverData to OverNode
    ///
    /// let n3 = tree.node(&id3);
    ///
    /// let paths: Vec<_> = n3
    ///     .paths_with(&mut dfs)
    ///     .map(|path_nodes| {
    ///         path_nodes
    ///             .map(|node| (*node.data(), node.depth()))
    ///             .collect::<Vec<_>>()
    ///     })
    ///     .collect();
    ///
    /// assert_eq!(
    ///     paths,
    ///     [
    ///         [(9, 3), (6, 2), (3, 1)],
    ///         [(10, 3), (7, 2), (3, 1)],
    ///         [(11, 3), (7, 2), (3, 1)]
    ///     ]
    /// );
    /// ```
    fn paths_with<T, O>(
        &'a self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = impl Iterator<Item = <O as Over>::NodeItem<'a, V, M, P>>>
    where
        O: Over<Enumeration = Val>,
        T: Traverser<O>,
    {
        let node_ptr = self.node_ptr();
        T::iter_ptr_with_storage(node_ptr.clone(), TraverserCore::storage_mut(traverser))
            .filter(|x| {
                let ptr: &NodePtr<V> = O::Enumeration::node_data(x);
                unsafe { &*ptr.ptr() }.next().is_empty()
            })
            .map(|x| {
                let ptr: &NodePtr<V> = O::Enumeration::node_data(&x);
                let iter = AncestorsIterPtr::new(node_ptr.clone(), ptr.clone());
                iter.map(|ptr: NodePtr<V>| {
                    O::Enumeration::from_element_ptr::<'a, V, M, P, O::NodeItem<'a, V, M, P>>(
                        self.col(),
                        ptr,
                    )
                })
            })
    }

    // traversal shorthands

    /// Returns an iterator of references to data of leaves of the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// Note that `leaves` is a shorthand of a chain of iterator methods over the more general [`walk_with`] method.
    /// This is demonstrated in the example below.
    ///
    /// [`walk_with`]: crate::NodeRef::walk_with
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // access the leaves in different orders that is determined by traversal
    ///
    /// let root = tree.root();
    ///
    /// let bfs_leaves: Vec<_> = root.leaves::<Bfs>().copied().collect();
    /// assert_eq!(bfs_leaves, [5, 8, 9, 10, 11]);
    ///
    /// let dfs_leaves: Vec<_> = root.leaves::<Dfs>().copied().collect();
    /// assert_eq!(dfs_leaves, [8, 5, 9, 10, 11]);
    ///
    /// // get the leaves from any node
    ///
    /// let n3 = tree.node(&id3);
    /// let leaves: Vec<_> = n3.leaves::<PostOrder>().copied().collect();
    /// assert_eq!(leaves, [9, 10, 11]);
    ///
    /// // ALTERNATIVELY: get the leaves with walk_with
    ///
    /// let mut tr = Traversal.bfs().over_nodes(); // we need Node to filter leaves
    ///
    /// let bfs_leaves: Vec<_> = root
    ///     .walk_with(&mut tr)
    ///     .filter(|x| x.is_leaf())
    ///     .map(|x| *x.data())
    ///     .collect();
    /// assert_eq!(bfs_leaves, [5, 8, 9, 10, 11]);
    /// ```
    fn leaves<T>(&'a self) -> impl Iterator<Item = &'a V::Item>
    where
        T: Traverser<OverData>,
    {
        T::iter_ptr_with_owned_storage(self.node_ptr().clone())
            .filter(|x: &NodePtr<V>| unsafe { &*x.ptr() }.next().is_empty())
            .map(|x: NodePtr<V>| {
                <OverData as Over>::Enumeration::from_element_ptr::<'a, V, M, P, &'a V::Item>(
                    self.col(),
                    x,
                )
            })
    }

    /// Returns an iterator of leaves of the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// Note that `leaves` is a shorthand of a chain of iterator methods over the more general [`walk_with`] method.
    /// This is demonstrated in the example below.
    ///
    /// [`walk_with`]: crate::NodeRef::walk_with
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.grow([4, 5]);
    ///
    /// tree.node_mut(&id4).push(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.grow([6, 7]);
    ///
    /// tree.node_mut(&id6).push(9);
    /// tree.node_mut(&id7).extend([10, 11]);
    ///
    /// // access leaves with re-usable traverser
    ///
    /// let mut bfs = Traversal.bfs();
    /// assert_eq!(
    ///     tree.root().leaves_with(&mut bfs).collect::<Vec<_>>(),
    ///     [&5, &8, &9, &10, &11]
    /// );
    /// assert_eq!(
    ///     tree.node(&id3).leaves_with(&mut bfs).collect::<Vec<_>>(),
    ///     [&9, &10, &11]
    /// );
    ///
    /// // access leaf nodes instead of data
    ///
    /// let mut dfs = Traversal.dfs().over_nodes();
    ///
    /// let root = tree.root();
    /// let mut leaves = root.leaves_with(&mut dfs);
    ///
    /// let leaf: Node<_> = leaves.next().unwrap();
    /// assert!(leaf.is_leaf());
    /// assert_eq!(leaf.data(), &8);
    /// assert_eq!(leaf.parent(), Some(tree.node(&id4)));
    ///
    /// // add depth and/or sibling-idx to the iteration items
    ///
    /// let mut dfs = Traversal.dfs().over_nodes().with_depth().with_sibling_idx();
    /// let mut leaves = root.leaves_with(&mut dfs);
    /// let (depth, sibling_idx, leaf) = leaves.next().unwrap();
    /// assert_eq!(depth, 3);
    /// assert_eq!(sibling_idx, 0);
    /// assert_eq!(leaf.data(), &8);
    /// ```
    fn leaves_with<T, O>(
        &'a self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        O: Over,
        T: Traverser<O>,
    {
        T::iter_ptr_with_storage(self.node_ptr().clone(), traverser.storage_mut())
            .filter(|x| {
                let ptr: &NodePtr<V> = O::Enumeration::node_data(x);
                unsafe { &*ptr.ptr() }.next().is_empty()
            })
            .map(|x| {
                O::Enumeration::from_element_ptr::<'a, V, M, P, O::NodeItem<'a, V, M, P>>(
                    self.col(),
                    x,
                )
            })
    }

    /// Returns an iterator of node indices.
    ///
    /// The order of the indices is determined by the generic [`Traverser`] parameter `T`.
    ///
    /// # See also
    ///
    /// Note that tree traversing methods typically allocate a temporary data structure that is dropped once the
    /// iterator is dropped.
    /// In use cases where we repeatedly iterate using any of the **walk** methods over different nodes or different
    /// trees, we can avoid the allocation by creating the traverser only once and using [`indices_with`] methods instead.
    /// This method additionally allow for yielding node depths and sibling indices in addition to node indices.
    ///
    /// [`indices_with`]: crate::NodeRef::indices_with
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      0
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     2
    /// //  ╱ ╲   ╱ ╲
    /// // 3   4 5   6
    /// // |     |  ╱ ╲
    /// // 7     8 9  10
    ///
    /// let mut a = DynTree::<i32>::new(0);
    /// let [a1, a2] = a.root_mut().grow([1, 2]);
    /// let [a3, _] = a.node_mut(&a1).grow([3, 4]);
    /// a.node_mut(&a3).push(7);
    /// let [a5, a6] = a.node_mut(&a2).grow([5, 6]);
    /// a.node_mut(&a5).push(8);
    /// a.node_mut(&a6).extend([9, 10]);
    ///
    /// // collect indices in breadth-first order
    ///
    /// let a0 = a.root();
    /// let bfs_indices: Vec<_> = a0.indices::<Bfs>().collect();
    ///
    /// assert_eq!(a.node(&bfs_indices[0]).data(), &0);
    /// assert_eq!(a.node(&bfs_indices[1]).data(), &1);
    /// assert_eq!(a.node(&bfs_indices[2]).data(), &2);
    /// assert_eq!(a.node(&bfs_indices[3]).data(), &3);
    ///
    /// // collect indices in depth-first order
    /// // we may also re-use a traverser
    ///
    /// let mut t = Traversal.dfs();
    ///
    /// let a0 = a.root();
    /// let dfs_indices: Vec<_> = a0.indices_with(&mut t).collect();
    ///
    /// assert_eq!(a.node(&dfs_indices[0]).data(), &0);
    /// assert_eq!(a.node(&dfs_indices[1]).data(), &1);
    /// assert_eq!(a.node(&dfs_indices[2]).data(), &3);
    /// assert_eq!(a.node(&dfs_indices[3]).data(), &7);
    /// ```
    fn indices<T>(&self) -> impl Iterator<Item = NodeIdx<V>>
    where
        T: Traverser<OverData>,
        V: 'static,
    {
        let node_ptr = self.node_ptr();
        let state = self.col().memory_state();
        T::iter_ptr_with_owned_storage(node_ptr.clone())
            .map(move |x: NodePtr<V>| NodeIdx(orx_selfref_col::NodeIdx::new(state, &x)))
    }

    /// Returns an iterator of node indices.
    ///
    /// The order of the indices is determined by the generic [`Traverser`] parameter `T` of the given `traverser`.
    ///
    /// Depending on the traverser type, the iterator might yield:
    ///
    /// * NodeIdx
    /// * (depth, NodeIdx)
    /// * (sibling_idx, NodeIdx)
    /// * (depth, sibling_idx, NodeIdx)
    ///
    /// # See also
    ///
    /// Note that tree traversing methods typically allocate a temporary data structure that is dropped once the
    /// iterator is dropped.
    /// In use cases where we repeatedly iterate using any of the **walk** methods over different nodes or different
    /// trees, we can avoid the allocation by creating the traverser only once and using [`indices_with`] methods instead.
    /// This method additionally allow for yielding node depths and sibling indices in addition to node indices.
    ///
    /// [`indices_with`]: crate::NodeRef::indices_with
    fn indices_with<'b, T, O>(
        &self,
        traverser: &'b mut T,
    ) -> impl Iterator<Item = <O::Enumeration as Enumeration>::Item<NodeIdx<V>>>
    where
        O: Over,
        T: Traverser<O>,
        V: 'static,
        Self: Sized,
    {
        let node_ptr = self.node_ptr();
        let state = self.col().memory_state();
        T::iter_ptr_with_storage(node_ptr.clone(), traverser.storage_mut()).map(move |x| {
            <O::Enumeration as Enumeration>::map_node_data(x, |ptr: NodePtr<V>| {
                NodeIdx(orx_selfref_col::NodeIdx::new(state, &ptr))
            })
        })
    }
}
