use crate::{
    helpers::{Col, N},
    memory::MemoryPolicy,
    pinned_storage::PinnedStorage,
    traversal::{over::OverItem, traverser_core::TraverserCore, Over, OverData, OverNode},
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
    /// assert_eq!(tree.root().unwrap().is_leaf(), true); // both root & leaf
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// let [id2, id3] = root.grow([2, 3]);
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// let [id4, id5] = n2.grow([4, 5]);
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// let [id6] = n3.grow([6]);
    ///
    /// // walk over any subtree rooted at a selected node
    /// // with different traversals
    ///
    /// assert_eq!(tree.root().unwrap().is_leaf(), false);
    /// assert_eq!(id2.node(&tree).is_leaf(), false);
    /// assert_eq!(id3.node(&tree).is_leaf(), false);
    ///
    /// assert_eq!(id4.node(&tree).is_leaf(), true);
    /// assert_eq!(id5.node(&tree).is_leaf(), true);
    /// assert_eq!(id6.node(&tree).is_leaf(), true);
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
    /// let [id_h, id_i, id_j] = g.grow(['h', 'i', 'j']);
    ///
    /// // check sibling positions
    ///
    /// let root = tree.root().unwrap();
    /// assert_eq!(root.sibling_position(), 0);
    ///
    /// for (i, node) in root.children().enumerate() {
    ///     assert_eq!(node.sibling_position(), i);
    /// }
    ///
    /// assert_eq!(id_h.node(&tree).sibling_position(), 0);
    /// assert_eq!(id_i.node(&tree).sibling_position(), 1);
    /// assert_eq!(id_j.node(&tree).sibling_position(), 2);
    /// ```
    fn sibling_position(&self) -> usize {
        let parent = self.node().prev().get().map(|ptr| unsafe { ptr.node() });
        parent
            .map(|parent| {
                let ptr = self.node_ptr();
                let mut children = parent.next().children_ptr();
                children.position(|x| x == ptr).expect("this node exists")
            })
            .unwrap_or(0)
    }

    // traversal

    /// Creates an iterator that yields references to data of all nodes belonging to the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the generic [`Traverser`] parameter `T`.
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
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
    /// // walk over any subtree rooted at a selected node
    /// // with different traversals
    ///
    /// let root = tree.root().unwrap();
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    ///
    /// let n3 = id3.node(&tree);
    /// let dfs: Vec<_> = n3.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);
    ///
    /// let n2 = id2.node(&tree);
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
    /// // create the traverser 'dfs' only once, use it many times
    /// // to walk over references, mutable references or removed values
    /// // without additional allocation
    ///
    /// let mut dfs = Dfs::default();
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = root.walk_with(&mut dfs).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
    ///
    /// let mut n7 = id7.node_mut(&mut tree);
    /// for x in n7.walk_mut_with(&mut dfs) {
    ///     *x += 100;
    /// }
    /// let values: Vec<_> = tree.root().unwrap().walk_with(&mut dfs).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 107, 110, 111]);
    ///
    /// let n3 = id3.node_mut(&mut tree);
    /// let removed: Vec<_> = n3.into_walk_with(&mut dfs).collect();
    /// assert_eq!(removed, [3, 6, 9, 107, 110, 111]);
    ///
    /// let remaining: Vec<_> = tree.root().unwrap().walk_with(&mut dfs).copied().collect();
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
    /// // create the traverser 'bfs' iterator
    /// // to walk over nodes rather than data
    ///
    /// let mut bfs = Bfs::default().over_nodes();
    /// // OR: Bfs::<OverNode>::new();
    ///
    /// let n7 = id7.node(&tree);
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
    /// let n3 = id3.node(&tree);
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

    // traversal shorthands

    fn leaves<T>(&'a self) -> impl Iterator<Item = Node<'a, V, M, P>>
    where
        T: Traverser<OverData>,
        Self: Sized,
        V::Item: Clone,
    {
        // core::iter::empty()
        <T::IntoOver<OverNode> as TraverserCore<OverNode>>::iter_with_owned_storage::<V, M, P>(self)
            .filter(|x| x.is_leaf())
        // .map(|x| x.data())
        // self.walk::<T::IntoOver<OverNode>>().filter(|x| x.is_leaf())
    }
}

#[test]
fn abc() {
    use crate::*;
    use alloc::vec::Vec;
    use std::dbg;

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

    // walk over any subtree rooted at a selected node
    // with different traversals

    let mut tr = Traversal.bfs().over_nodes();
    let t = &mut tr;

    let root = tree.root().unwrap();
    let leaves: Vec<_> = root
        .walk_with(t)
        .filter(|x| x.is_leaf())
        .map(|x| *x.data())
        .collect();
    std::println!("\n\n{:?}\n\n", leaves);

    let all: Vec<_> = root.walk_with(t).map(|x| *x.data()).collect();
    assert_eq!(all, []);
}
