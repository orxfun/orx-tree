use crate::{
    helpers::N,
    iter::{Bfs, Dfs, IterKindCore, IterOver, NodeVal, NodeValueData},
    tree_variant::RefsChildren,
    Node, TreeVariant,
};
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait NodeRefCore<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    fn col(&self) -> &SelfRefCol<V, M, P>;

    fn node_ptr(&self) -> &NodePtr<V>;

    #[inline(always)]
    fn node(&self) -> &N<V> {
        unsafe { &*self.node_ptr().ptr() }
    }
}

impl<'a, V, M, P, X> NodeRef<'a, V, M, P> for X
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    X: NodeRefCore<'a, V, M, P>,
{
}

/// Reference to a tree node.
pub trait NodeRef<'a, V, M, P>: NodeRefCore<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
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
    /// let a = root.push(1);
    /// let node = tree.node(&a).unwrap();
    /// assert_eq!(node.data(), &1);
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
    /// let a = root.push(1);
    /// let b = root.push(2);
    /// assert_eq!(root.num_children(), 2);
    ///
    /// let mut node = tree.node_mut(&a).unwrap();
    /// node.extend([3, 4, 5, 6]);
    /// assert_eq!(node.num_children(), 4);
    ///
    /// assert_eq!(tree.node(&b).unwrap().num_children(), 0);
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
    /// let a = root.push('a');
    /// root.push('b');
    ///
    /// let mut node_a = tree.node_mut(&a).unwrap();
    /// node_a.extend(['c', 'd', 'e']);
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
    /// let a = root.push('a');
    /// root.push('b');
    ///
    /// let mut node_a = tree.node_mut(&a).unwrap();
    /// node_a.extend(['c', 'd', 'e']);
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
    /// // build the following tree using child_mut and parent_mut:
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
    /// let values: Vec<_> = tree.root().unwrap().dfs().copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// // dfs from any node
    ///
    /// let root = tree.root().unwrap();
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = n3.dfs().copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    ///
    /// let idx6 = &n3_children_idx[0];
    /// let n6 = tree.node(idx6).unwrap();
    /// let values: Vec<_> = n6.dfs().copied().collect();
    /// assert_eq!(values, [6, 9]);
    /// ```
    fn dfs(&self) -> Dfs<NodeVal<NodeValueData>, V, M, P> {
        Dfs::new(self.col(), self.node_ptr().clone())
    }

    /// Creates a depth first search iterator over the data of the nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` which yields [`data`] of each traversed node.
    ///
    /// See also [`dfs_over_using`] for variants yielding different values for each traversed node.
    ///
    /// # dfs & dfs_using
    ///
    /// `dfs_using` differs from [`dfs`] in the following:
    /// * Depth first search requires a stack (Vec) to be allocated.
    /// * Every time `node.dfs()` is called, a new vector is allocated, and it is dropped once the iterator is consumed.
    /// * `node.dfs_using`, on the other hand, requires a mutable reference to a vector to be used throughout the iteration.
    ///   Therefore, it does not require to allocate any intermediate data.
    ///   This fits best to situations where:
    ///   * we want to allocate as little as possible, and
    ///   * we repeatedly traverse over the tree, and hence, we re-use the same stack over and over without new allocations.
    ///
    /// [`dfs`]: crate::NodeRef::dfs
    /// [`data`]: crate::NodeRef::data
    /// [`dfs_over_using`]: crate::NodeRef::dfs_over_using
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
    /// // allocate the stack once
    ///
    /// let mut stack = vec![];
    ///
    /// // depth-first-search (dfs) from the root
    ///
    /// let values: Vec<_> = tree.root().unwrap().dfs_using(&mut stack).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// // dfs from any node
    ///
    /// let root = tree.root().unwrap();
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = n3.dfs_using(&mut stack).copied().collect();
    /// assert_eq!(values, [3, 6, 9, 7, 10, 11]);
    ///
    /// let idx6 = &n3_children_idx[0];
    /// let n6 = tree.node(idx6).unwrap();
    /// let values: Vec<_> = n6.dfs_using(&mut stack).copied().collect();
    /// assert_eq!(values, [6, 9]);
    /// ```
    fn dfs_using(
        &'a self,
        stack: &'a mut Vec<NodePtr<V>>,
    ) -> Dfs<'a, NodeVal<NodeValueData>, V, M, P, &'a mut Vec<NodePtr<V>>> {
        Dfs::new_with_queue(self.col(), self.node_ptr().clone(), stack)
    }

    /// Creates a depth first search iterator over different values of nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic type parameter:
    ///
    /// * [`OverData`] yields [`data`] of nodes (therefore, node.dfs_over::&lt;Data&gt;() is equivalent to node.dfs())
    /// * [`OverDepthData`] yields (depth, ['data']) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, ['data']) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
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
    /// let mut iter = root.dfs_over::<OverDepthSiblingData>();
    /// assert_eq!(iter.next(), Some((0, 0, &1))); // (depth, sibling idx, data)
    /// assert_eq!(iter.next(), Some((1, 0, &2)));
    /// assert_eq!(iter.next(), Some((2, 0, &4)));
    /// assert_eq!(iter.next(), Some((3, 0, &8)));
    /// assert_eq!(iter.next(), Some((2, 1, &5)));
    /// assert_eq!(iter.next(), Some((1, 1, &3))); // ...
    ///
    /// let all: Vec<(usize, usize, &i32)> = root.dfs_over::<OverDepthSiblingData>().collect();
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
    /// let nodes: Vec<(usize, usize, Node<_>)> = root.dfs_over::<OverDepthSiblingNode>().collect();
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
    fn dfs_over<K: IterOver>(&'a self) -> Dfs<'a, K::IterKind<'a, V, M, P>, V, M, P> {
        Dfs::new(self.col(), self.node_ptr().clone())
    }

    /// Creates a depth first search iterator over different values of nodes;
    /// also known as "pre-order traversal" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order,_NLR)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic type parameter:
    ///
    /// * [`OverData`] yields [`data`] of nodes (therefore, node.dfs_over_using::&lt;Data&gt;() is equivalent to node.dfs_using())
    /// * [`OverDepthData`] yields (depth, ['data']) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, ['data']) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// # dfs_over & dfs_over_using
    ///
    /// `dfs_over_using` differs from [`dfs_over`] in the following:
    /// * Depth first search requires a stack (Vec) to be allocated.
    /// * Every time `node.dfs_over()` is called, a new vector is allocated, and it is dropped once the iterator is consumed.
    /// * `node.dfs_over_using`, on the other hand, requires a mutable reference to a vector to be used throughout the iteration.
    ///   Therefore, it does not require to allocate any intermediate data.
    ///   This fits best to situations where:
    ///   * we want to allocate as little as possible, and
    ///   * we repeatedly traverse over the tree, and hence, we re-use the same stack over and over without new allocations.
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    /// [`dfs_over`]: crate::NodeRef::dfs_over
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
    /// // dfs over (depth, data)
    ///
    /// let mut stack = vec![]; // allocate stack only once
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = root.dfs_over_using::<OverDepthData>(&mut stack);
    /// assert_eq!(iter.next(), Some((0, &1)));
    /// assert_eq!(iter.next(), Some((1, &2)));
    /// assert_eq!(iter.next(), Some((2, &4)));
    /// assert_eq!(iter.next(), Some((3, &8)));
    /// assert_eq!(iter.next(), Some((2, &5))); // ...
    ///
    /// let depths: Vec<usize> = root.dfs_over_using::<OverDepthData>(&mut stack).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
    ///
    /// let values: Vec<i32> = root.dfs_over_using::<OverDepthData>(&mut stack).map(|x| *x.1).collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    /// ```
    fn dfs_over_using<K: IterOver>(
        &'a self,
        stack: &'a mut Vec<<K::IterKind<'a, V, M, P> as IterKindCore<'a, V, M, P>>::QueueElement>,
    ) -> Dfs<
        'a,
        K::IterKind<'a, V, M, P>,
        V,
        M,
        P,
        &'a mut Vec<<K::IterKind<'a, V, M, P> as IterKindCore<'a, V, M, P>>::QueueElement>,
    > {
        Dfs::new_with_queue(self.col(), self.node_ptr().clone(), stack)
    }

    // bfs

    /// Creates a breadth first search iterator over the data of the nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` which yields [`data`] of each traversed node.
    ///
    /// See also [`dfs_over`] for variants yielding different values for each traversed node.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`dfs_over`]: crate::NodeRef::dfs_over
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
    /// // breadth-first-search (bfs) from the root
    ///
    /// let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// // bfs from any node
    ///
    /// let root = tree.root().unwrap();
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = n3.bfs().copied().collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    ///
    /// let idx6 = &n3_children_idx[0];
    /// let n6 = tree.node(idx6).unwrap();
    /// let values: Vec<_> = n6.bfs().copied().collect();
    /// assert_eq!(values, [6, 9]);
    /// ```
    fn bfs(&self) -> Bfs<NodeVal<NodeValueData>, V, M, P> {
        Bfs::new(self.col(), self.node_ptr().clone())
    }

    /// Creates a breadth first search iterator over the data of the nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` which yields [`data`] of each traversed node.
    ///
    /// See also [`bfs_over_using`] for variants yielding different values for each traversed node.
    ///
    /// # bfs & bfs_using
    ///
    /// `bfs_using` differs from [`bfs`] in the following:
    /// * Depth first search requires a queue (VecDeque) to be allocated.
    /// * Every time `node.bfs()` is called, a new queue is allocated, and it is dropped once the iterator is consumed.
    /// * `node.bfs_using`, on the other hand, requires a mutable reference to a queue to be used throughout the iteration.
    ///   Therefore, it does not require to allocate any intermediate data.
    ///   This fits best to situations where:
    ///   * we want to allocate as little as possible, and
    ///   * we repeatedly traverse over the tree, and hence, we re-use the same queue over and over without new allocations.
    ///
    /// [`bfs`]: crate::NodeRef::bfs
    /// [`data`]: crate::NodeRef::data
    /// [`bfs_over_using`]: crate::NodeRef::bfs_over_using
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use std::collections::VecDeque;
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
    /// // allocate the queue once
    ///
    /// let mut queue = VecDeque::new();
    ///
    /// // breadth-first-search (dfs) from the root
    ///
    /// let values: Vec<_> = tree.root().unwrap().bfs_using(&mut queue).copied().collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// // bfs from any node
    ///
    /// let root = tree.root().unwrap();
    /// let n3 = root.child(1).unwrap();
    /// let values: Vec<_> = n3.bfs_using(&mut queue).copied().collect();
    /// assert_eq!(values, [3, 6, 7, 9, 10, 11]);
    ///
    /// let idx6 = &n3_children_idx[0];
    /// let n6 = tree.node(idx6).unwrap();
    /// let values: Vec<_> = n6.bfs_using(&mut queue).copied().collect();
    /// assert_eq!(values, [6, 9]);
    /// ```
    fn bfs_using(
        &'a self,
        queue: &'a mut VecDeque<NodePtr<V>>,
    ) -> Bfs<'a, NodeVal<NodeValueData>, V, M, P, &'a mut VecDeque<NodePtr<V>>> {
        Bfs::new_with_queue(self.col(), self.node_ptr().clone(), queue)
    }

    /// Creates a breadth first search iterator over different values of nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic type parameter:
    ///
    /// * [`OverData`] yields [`data`] of nodes (therefore, node.dfs_over::&lt;Data&gt;() is equivalent to node.dfs())
    /// * [`OverDepthData`] yields (depth, ['data']) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, ['data']) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
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
    /// let mut tree = DynTree::<i32>::new(1);
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
    /// let mut iter = root.bfs_over::<OverDepthSiblingData>();
    /// assert_eq!(iter.next(), Some((0, 0, &1))); // (depth, sibling idx, data)
    /// assert_eq!(iter.next(), Some((1, 0, &2)));
    /// assert_eq!(iter.next(), Some((1, 1, &3)));
    /// assert_eq!(iter.next(), Some((2, 0, &4)));
    /// assert_eq!(iter.next(), Some((2, 1, &5)));
    /// assert_eq!(iter.next(), Some((2, 0, &6))); // ...
    ///
    /// let all: Vec<(usize, usize, &i32)> = root.bfs_over::<OverDepthSiblingData>().collect();
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
    /// let nodes: Vec<(usize, usize, Node<_>)> = root.bfs_over::<OverDepthSiblingNode>().collect();
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
    fn bfs_over<K: IterOver>(&'a self) -> Bfs<'a, K::IterKind<'a, V, M, P>, V, M, P> {
        Bfs::new(self.col(), self.node_ptr().clone())
    }

    /// Creates a breadth first search iterator over different values of nodes.
    /// This traversal also known as "level-order" ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
    ///
    /// Return value is an `Iterator` with polymorphic element types which are determined by the generic type parameter:
    ///
    /// * [`OverData`] yields [`data`] of nodes (therefore, node.bfs_over_using::&lt;Data&gt;() is equivalent to node.bfs_using())
    /// * [`OverDepthData`] yields (depth, ['data']) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, ['data']) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// # bfs_over & bfs_over_using
    ///
    /// `bfs_over_using` differs from [`bfs_over`] in the following:
    /// * Depth first search requires a queue (VecDeque) to be allocated.
    /// * Every time `node.bfs_over()` is called, a new queue is allocated, and it is dropped once the iterator is consumed.
    /// * `node.bfs_over_using`, on the other hand, requires a mutable reference to a queue to be used throughout the iteration.
    ///   Therefore, it does not require to allocate any intermediate data.
    ///   This fits best to situations where:
    ///   * we want to allocate as little as possible, and
    ///   * we repeatedly traverse over the tree, and hence, we re-use the same queue over and over without new allocations.
    ///
    /// You may see below how to conveniently create iterators yielding possible element types using above-mentioned generic parameters.
    ///
    /// [`data`]: crate::NodeRef::data
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    /// [`bfs_over`]: crate::NodeRef::bfs_over
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    /// use orx_tree::iter::*;
    /// use std::collections::VecDeque;
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
    /// // dfs over (depth, data)
    ///
    /// let mut queue = VecDeque::new(); // allocate queue only once
    ///
    /// let root = tree.root().unwrap();
    ///
    /// let mut iter = root.bfs_over_using::<OverDepthData>(&mut queue);
    /// assert_eq!(iter.next(), Some((0, &1)));
    /// assert_eq!(iter.next(), Some((1, &2)));
    /// assert_eq!(iter.next(), Some((1, &3)));
    /// assert_eq!(iter.next(), Some((2, &4)));
    /// assert_eq!(iter.next(), Some((2, &5))); // ...
    ///
    /// let depths: Vec<usize> = root.bfs_over_using::<OverDepthData>(&mut queue).map(|x| x.0).collect();
    /// assert_eq!(depths, [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
    ///
    /// let values: Vec<i32> = root.bfs_over_using::<OverDepthData>(&mut queue).map(|x| *x.1).collect();
    /// assert_eq!(values, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    /// ```
    fn bfs_over_using<K: IterOver>(
        &'a self,
        queue: &'a mut VecDeque<
            <K::IterKind<'a, V, M, P> as IterKindCore<'a, V, M, P>>::QueueElement,
        >,
    ) -> Bfs<
        'a,
        K::IterKind<'a, V, M, P>,
        V,
        M,
        P,
        &'a mut VecDeque<<K::IterKind<'a, V, M, P> as IterKindCore<'a, V, M, P>>::QueueElement>,
    > {
        Bfs::new_with_queue(self.col(), self.node_ptr().clone(), queue)
    }
}
