use crate::{
    helpers::{Col, N},
    memory::MemoryPolicy,
    pinned_storage::PinnedStorage,
    traversal::{enumerations::Val, over::OverItem, Over, OverData},
    tree_variant::RefsChildren,
    Node, NodeIdx, Traverser, TreeVariant,
};
use orx_selfref_col::NodePtr;

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
        NodeIdx::new(self.col().memory_state(), self.node_ptr())
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
    /// let mut root = tree.root_mut().unwrap();
    /// assert!(root.is_root());
    ///
    /// root.extend(['a', 'b']);
    /// for node in root.children() {
    ///     assert!(!node.is_root());
    /// }
    /// ```
    fn is_root(&self) -> bool {
        self.node().prev().get().is_none()
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
    /// let mut root = tree.root_mut().unwrap();
    /// assert_eq!(root.data(), &0);
    ///
    /// let [id_a] = root.grow([1]);
    /// let a = id_a.node(&tree);
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
    /// let mut root = tree.root_mut().unwrap();
    /// assert_eq!(root.num_children(), 0);
    ///
    /// let [id_a, id_b] = root.grow([1, 2]);
    /// assert_eq!(root.num_children(), 2);
    ///
    /// let mut node = id_a.node_mut(&mut tree);
    /// node.push(3);
    /// node.extend([4, 5, 6]);
    /// assert_eq!(node.num_children(), 4);
    ///
    /// assert_eq!(id_b.node(&tree).num_children(), 0);
    /// ```
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
    /// let mut root = tree.root_mut().unwrap();
    /// let [id_a] = root.grow(['a']);
    /// root.push('b');
    ///
    /// let mut a = id_a.node_mut(&mut tree);
    /// a.extend(['c', 'd', 'e']);
    ///
    /// // iterate over children of nodes
    ///
    /// let root = tree.root().unwrap();
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
    /// let mut root = tree.root_mut().unwrap();
    /// let [id_a] = root.grow(['a']);
    /// root.push('b');
    ///
    /// let mut a = id_a.node_mut(&mut tree);
    /// a.extend(['c', 'd', 'e']);
    ///
    /// // use child to access lower level nodes
    ///
    /// let root = tree.root().unwrap();
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
    /// let mut root = tree.root_mut().unwrap();
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
    /// returns None if this is the root node.
    ///
    /// **O(S)** where S is the number of siblings; i.e.,
    /// requires linear search over the children of the parent of this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // build the following tree using into_child_mut and into_parent_mut:
    /// // r
    /// // +-- a
    /// // |   +-- c, d, e
    /// // |
    /// // +-- b
    /// //     +-- f, g
    /// //            +-- h, i, j
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
    /// // check data - breadth first
    ///
    /// let root = tree.root().unwrap();
    /// assert_eq!(root.sibling_position(), None);
    ///
    /// let b = root.child(1).unwrap();
    /// assert_eq!(b.sibling_position(), Some(1));
    ///
    /// let g = b.child(1).unwrap();
    /// assert_eq!(g.sibling_position(), Some(1));
    ///
    /// let j = g.child(2).unwrap();
    /// assert_eq!(j.sibling_position(), Some(2));
    /// ```
    fn sibling_position(&self) -> Option<usize> {
        let parent = self.node().prev().get().map(|ptr| unsafe { ptr.node() });

        parent.map(|parent| {
            let ptr = self.node_ptr();
            let mut children = parent.next().children_ptr();
            children.position(|x| x == ptr).expect("this node exists")
        })
    }

    // dfs

    /// Creates a depth first search iterator over the data of the nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` which yields [`data`] of each traversed node.
    ///
    /// See also [`dfs_over`] for variants yielding different values for each traversed node.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`dfs_over`]: crate::NodeRef::dfs_over
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
    /// // traversal from any node
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = root.dfs().copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = n3.dfs().copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    ///
    /// let n7 = id7.node(&tree);
    /// let values: Vec<_> = n7.dfs().copied().collect();
    /// assert_eq!(values, [7, 10, 11]);
    /// ```
    fn dfs(&'a self) -> impl Iterator<Item = &'a V::Item> {
        use crate::traversal::depth_first::{iter_ptr::DfsIterPtr, iter_ref::DfsIterRef};
        let root = self.node_ptr().clone();
        let iter = DfsIterPtr::<_, Val>::from((Default::default(), root));
        DfsIterRef::<'_, _, M, P, _, _, _>::from((self.col(), iter))
    }

    /// Creates a depth first search iterator over different values of nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic [`Over`] type parameter `O`.
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
    /// // dfs over data
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let values: Vec<i32> = root.dfs_over::<OverData>().copied().collect(); // or simply dfs()
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// // dfs over (depth, data)
    ///
    /// let mut iter = root.dfs_over::<OverDepthData>();
    /// assert_eq!(iter.next(), Some((0, &1)));
    /// assert_eq!(iter.next(), Some((1, &2)));
    /// assert_eq!(iter.next(), Some((2, &4)));
    /// assert_eq!(iter.next(), Some((3, &8)));
    /// assert_eq!(iter.next(), Some((2, &5))); // ...
    ///
    /// let all: Vec<(usize, &i32)> = root.dfs_over::<OverDepthData>().collect();
    ///
    /// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
    ///
    /// let values: Vec<i32> = all.iter().map(|x| *x.1).collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// // dfs over (depth, sibling index, data)
    ///
    /// let mut iter = root.dfs_over::<OverDepthSiblingIdxData>();
    /// assert_eq!(iter.next(), Some((0, 0, &1))); // (depth, sibling idx, data)
    /// assert_eq!(iter.next(), Some((1, 0, &2)));
    /// assert_eq!(iter.next(), Some((2, 0, &4)));
    /// assert_eq!(iter.next(), Some((3, 0, &8)));
    /// assert_eq!(iter.next(), Some((2, 1, &5)));
    /// assert_eq!(iter.next(), Some((1, 1, &3))); // ...
    ///
    /// let all: Vec<(usize, usize, &i32)> = root.dfs_over::<OverDepthSiblingIdxData>().collect();
    ///
    /// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
    ///
    /// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
    /// assert_eq!(sibling_indices, [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]);
    ///
    /// let values: Vec<i32> = all.iter().map(|x| *x.2).collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// // dfs over nodes OR (depth, node) pairs OR (depth, sibling index, node) tuples
    ///
    /// let nodes: Vec<Node<_>> = root.dfs_over::<OverNode>().collect();
    /// for (node, expected_value) in nodes.iter().zip(&values) {
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    ///
    /// let nodes: Vec<(usize, Node<_>)> = root.dfs_over::<OverDepthNode>().collect();
    /// for ((depth, node), (expected_depth, expected_value)) in
    ///     nodes.iter().zip(depths.iter().zip(&values))
    /// {
    ///     assert_eq!(depth, expected_depth);
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    ///
    /// let nodes: Vec<(usize, usize, Node<_>)> = root.dfs_over::<OverDepthSiblingIdxNode>().collect();
    /// for ((depth, sibling_idx, node), (expected_depth, (expected_sibling_idx, expected_value))) in
    ///     nodes
    ///         .iter()
    ///         .zip(depths.iter().zip(sibling_indices.iter().zip(&values)))
    /// {
    ///     assert_eq!(depth, expected_depth);
    ///     assert_eq!(sibling_idx, expected_sibling_idx);
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    /// ```
    fn dfs_over<O: Over>(&'a self) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> {
        use crate::traversal::depth_first::{iter_ptr::DfsIterPtr, iter_ref::DfsIterRef};
        let root = self.node_ptr().clone();
        let iter = DfsIterPtr::<_, O::Enumeration>::from((Default::default(), root));
        DfsIterRef::from((self.col(), iter))
    }

    // bfs

    /// Creates a breadth first search iterator over the data of the nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` which yields [`data`] of each traversed node.
    ///
    /// See also [`bfs_over`] for variants yielding different values for each traversed node.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`bfs_over`]: crate::NodeRef::bfs_over
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
    /// // traversal from any node
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = root.bfs().copied().collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = n3.bfs().copied().collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    ///
    /// let n7 = id7.node(&tree);
    /// let values: Vec<_> = n7.bfs().copied().collect();
    /// assert_eq!(values, [7, 10, 11]);
    /// ```
    fn bfs(&'a self) -> impl Iterator<Item = &'a V::Item> {
        use crate::traversal::breadth_first::{iter_ptr::BfsIterPtr, iter_ref::BfsIterRef};
        let root = self.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val>::from((Default::default(), root));
        BfsIterRef::<'_, _, M, P, _, _, _>::from((self.col(), iter))
    }

    /// Creates a breadth first search iterator over different values of nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic [`Over`] type parameter `O`.
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
    /// // bfs over data
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let values: Vec<i32> = root.bfs_over::<OverData>().copied().collect(); // or simply bfs()
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// // bfs over (depth, data)
    ///
    /// let mut iter = root.bfs_over::<OverDepthData>();
    /// assert_eq!(iter.next(), Some((0, &1))); // (depth, data)
    /// assert_eq!(iter.next(), Some((1, &2)));
    /// assert_eq!(iter.next(), Some((1, &3)));
    /// assert_eq!(iter.next(), Some((2, &4)));
    /// assert_eq!(iter.next(), Some((2, &5)));
    /// assert_eq!(iter.next(), Some((2, &6))); // ...
    ///
    /// let all: Vec<(usize, &i32)> = root.bfs_over::<OverDepthData>().collect();
    ///
    /// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
    ///
    /// let values: Vec<i32> = all.iter().map(|x| *x.1).collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// // bfs over (depth, sibling index, data)
    ///
    /// let mut iter = root.bfs_over::<OverDepthSiblingIdxData>();
    /// assert_eq!(iter.next(), Some((0, 0, &1))); // (depth, sibling idx, data)
    /// assert_eq!(iter.next(), Some((1, 0, &2)));
    /// assert_eq!(iter.next(), Some((1, 1, &3)));
    /// assert_eq!(iter.next(), Some((2, 0, &4)));
    /// assert_eq!(iter.next(), Some((2, 1, &5)));
    /// assert_eq!(iter.next(), Some((2, 0, &6))); // ...
    ///
    /// let all: Vec<(usize, usize, &i32)> = root.bfs_over::<OverDepthSiblingIdxData>().collect();
    ///
    /// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
    ///
    /// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
    /// assert_eq!(sibling_indices, [0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1]);
    ///
    /// let values: Vec<i32> = all.iter().map(|x| *x.2).collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// // bfs over nodes OR (depth, node) pairs OR (depth, sibling index, node) tuples
    ///
    /// let nodes: Vec<Node<_>> = root.bfs_over::<OverNode>().collect();
    /// for (node, expected_value) in nodes.iter().zip(&values) {
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    ///
    /// let nodes: Vec<(usize, Node<_>)> = root.bfs_over::<OverDepthNode>().collect();
    /// for ((depth, node), (expected_depth, expected_value)) in
    ///     nodes.iter().zip(depths.iter().zip(&values))
    /// {
    ///     assert_eq!(depth, expected_depth);
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    ///
    /// let nodes: Vec<(usize, usize, Node<_>)> = root.bfs_over::<OverDepthSiblingIdxNode>().collect();
    /// for ((depth, sibling_idx, node), (expected_depth, (expected_sibling_idx, expected_value))) in
    ///     nodes
    ///         .iter()
    ///         .zip(depths.iter().zip(sibling_indices.iter().zip(&values)))
    /// {
    ///     assert_eq!(depth, expected_depth);
    ///     assert_eq!(sibling_idx, expected_sibling_idx);
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    /// ```
    fn bfs_over<O: Over>(&'a self) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> {
        use crate::traversal::breadth_first::{iter_ptr::BfsIterPtr, iter_ref::BfsIterRef};
        let root = self.node_ptr().clone();
        let iter = BfsIterPtr::<_, O::Enumeration>::from((Default::default(), root));
        BfsIterRef::from((self.col(), iter))
    }

    // post-order

    /// Creates an iterator for post-order traversal rooted at this node
    /// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
    ///
    /// An important property of post-order traversal is that nodes are yield after their all descendants are
    /// yield; and hence, the root (this) node will be yield at last.
    /// Among other reasons, this makes post-order traversal very useful for pruning or removing nodes from trees.
    ///
    /// Return value is an `Iterator` which yields [`data`] of each traversed node.
    ///
    /// See also [`post_order_over`] for variants yielding different values for each traversed node.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`post_order_over`]: crate::NodeRef::post_order_over
    ///
    /// # Allocation
    ///
    /// Note that post  order traversal requires a vector (alloc::vec::Vec) to be allocated, with a length equal to
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
    /// // post-order traversal from any node
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = root.post_order().copied().collect();
    /// assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = n3.post_order().copied().collect();
    /// assert_eq!(values, [9, 6, 10, 11, 7, 3]);
    ///
    /// let n7 = id7.node(&tree);
    /// let values: Vec<_> = n7.post_order().copied().collect();
    /// assert_eq!(values, [10, 11, 7]);
    /// ```
    fn post_order(&'a self) -> impl Iterator<Item = &'a V::Item> {
        use crate::traversal::post_order::{
            iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
        };
        let root = self.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val>::from((Default::default(), root));
        PostOrderIterRef::<'_, _, M, P, _, _, _>::from((self.col(), iter))
    }

    /// Creates an iterator for post-order traversal rooted at this node over different values of the nodes
    /// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
    ///
    /// An important property of post-order traversal is that nodes are yield after their all descendants are
    /// yield; and hence, the root (this) node will be yield at last.
    /// Among other reasons, this makes post-order traversal very useful for pruning or removing nodes from trees.
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic [`Over`] type parameter `O`.
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// # Allocation
    ///
    /// Note that post  order traversal requires a vector (alloc::vec::Vec) to be allocated, with a length equal to
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
    /// // post-order over data
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let values: Vec<i32> = root.post_order_over::<OverData>().copied().collect(); // or simply bfs()
    /// assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
    ///
    /// // post-order over (depth, data)
    ///
    /// let mut iter = root.post_order_over::<OverDepthData>();
    /// assert_eq!(iter.next(), Some((3, &8))); // (depth, data)
    /// assert_eq!(iter.next(), Some((2, &4)));
    /// assert_eq!(iter.next(), Some((2, &5)));
    /// assert_eq!(iter.next(), Some((1, &2)));
    /// assert_eq!(iter.next(), Some((3, &9)));
    /// assert_eq!(iter.next(), Some((2, &6))); // ...
    ///
    /// let all: Vec<(usize, &i32)> = root.post_order_over::<OverDepthData>().collect();
    ///
    /// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
    /// assert_eq!(depths, [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]);
    ///
    /// let values: Vec<i32> = all.iter().map(|x| *x.1).collect();
    /// assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
    ///
    /// // post-order over (depth, sibling index, data)
    ///
    /// let mut iter = root.post_order_over::<OverDepthSiblingIdxData>();
    /// assert_eq!(iter.next(), Some((3, 0, &8))); // (depth, sibling idx, data)
    /// assert_eq!(iter.next(), Some((2, 0, &4)));
    /// assert_eq!(iter.next(), Some((2, 1, &5)));
    /// assert_eq!(iter.next(), Some((1, 0, &2)));
    /// assert_eq!(iter.next(), Some((3, 0, &9)));
    /// assert_eq!(iter.next(), Some((2, 0, &6))); // ...
    ///
    /// let all: Vec<(usize, usize, &i32)> = root.post_order_over::<OverDepthSiblingIdxData>().collect();
    ///
    /// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
    /// assert_eq!(depths, [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]);
    ///
    /// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
    /// assert_eq!(sibling_indices, [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]);
    ///
    /// let values: Vec<i32> = all.iter().map(|x| *x.2).collect();
    /// assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
    ///
    /// // post-order over nodes OR (depth, node) pairs OR (depth, sibling index, node) tuples
    ///
    /// let nodes: Vec<Node<_>> = root.post_order_over::<OverNode>().collect();
    /// for (node, expected_value) in nodes.iter().zip(&values) {
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    ///
    /// let nodes: Vec<(usize, Node<_>)> = root.post_order_over::<OverDepthNode>().collect();
    /// for ((depth, node), (expected_depth, expected_value)) in
    ///     nodes.iter().zip(depths.iter().zip(&values))
    /// {
    ///     assert_eq!(depth, expected_depth);
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    ///
    /// let nodes: Vec<(usize, usize, Node<_>)> =
    ///     root.post_order_over::<OverDepthSiblingIdxNode>().collect();
    /// for ((depth, sibling_idx, node), (expected_depth, (expected_sibling_idx, expected_value))) in
    ///     nodes
    ///         .iter()
    ///         .zip(depths.iter().zip(sibling_indices.iter().zip(&values)))
    /// {
    ///     assert_eq!(depth, expected_depth);
    ///     assert_eq!(sibling_idx, expected_sibling_idx);
    ///     assert_eq!(node.data(), expected_value);
    ///     assert!(node.num_children() <= 2);
    /// }
    /// ```
    fn post_order_over<O: Over>(&'a self) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> {
        use crate::traversal::post_order::{
            iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
        };
        let root = self.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, O::Enumeration>::from((Default::default(), root));
        PostOrderIterRef::from((self.col(), iter))
    }

    // traversal

    fn walk<T>(&'a self) -> impl Iterator<Item = &'a V::Item>
    where
        T: Traverser<OverData> + 'a,
        Self: Sized,
    {
        T::iter_with_owned_storage::<V, M, P>(self)
    }
}

#[test]
fn abc() {
    use crate::traversal::*;
    use crate::*;
    use alloc::vec::Vec;

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11

    let mut tree = DynTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();
    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree).push(8);

    let mut n3 = id3.node_mut(&mut tree);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree).push(9);
    id7.node_mut(&mut tree).extend([10, 11]);

    // traversal from any node

    let root = tree.root().unwrap();
    let values: Vec<_> = root.walk::<Dfs>().copied().collect();
    assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    let n3 = id3.node(&tree);
    let values: Vec<_> = n3.walk::<Dfs>().copied().collect();
    assert_eq!(values, [3, 6, 9, 7, 10, 11]);

    let n7 = id7.node(&tree);
    let values: Vec<_> = n7.walk::<Dfs>().copied().collect();
    assert_eq!(values, [7, 10, 11]);
}
