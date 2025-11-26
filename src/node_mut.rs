use crate::{
    Node, NodeIdx, NodeRef, PostOrder, SubTree, Traverser, Tree, TreeVariant,
    aliases::{Col, N},
    iter::{ChildrenMutIter, CustomWalkIterPtr},
    memory::{Auto, MemoryPolicy},
    node_ref::NodeRefCore,
    pinned_storage::{PinnedStorage, SplitRecursive},
    subtrees::{MovedSubTree, SubTreeCore},
    subtrees_within::SubTreeWithin,
    traversal::{
        Over, OverData, OverMut,
        enumeration::Enumeration,
        enumerations::Val,
        over::OverPtr,
        over_mut::{OverItemInto, OverItemMut},
        post_order::iter_ptr::PostOrderIterPtr,
        traverser_core::TraverserCore,
    },
    tree_node_idx::INVALID_IDX_ERROR,
    tree_variant::RefsChildren,
};
use alloc::vec::Vec;
use core::{fmt::Debug, marker::PhantomData};
use orx_selfref_col::{NodePtr, Refs};

/// A marker trait determining the mutation flexibility of a mutable node.
pub trait NodeMutOrientation: 'static {}

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

/// Side of a sibling node relative to a particular node within the children collection.
#[derive(Clone, Copy, Debug)]
pub enum Side {
    /// To the left of this node.
    Left,
    /// To the right of this node.
    Right,
}

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
    fn col(&self) -> &'a Col<V, M, P> {
        let x = self.col as *const Col<V, M, P>;
        unsafe { &*x }
    }

    #[inline(always)]
    fn node_ptr(&self) -> NodePtr<V> {
        self.node_ptr
    }
}

impl<'a, V, M, P, MO> NodeMut<'a, V, M, P, MO>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    MO: NodeMutOrientation,
{
    /// Returns the mutable `root` node of the tree that this node belongs to.
    ///
    /// Note that if this node is the root of its tree, this method will return itself.
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// let [id10, _] = tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // reach back to root from any node
    ///
    /// let mut n1 = tree.root_mut();
    /// let mut root = n1.root_mut();
    /// *root.data_mut() += 100;
    /// assert_eq!(tree.root().data(), &101);
    ///
    /// let mut n4 = tree.node_mut(id4);
    /// *n4.root_mut().data_mut() += 100;
    /// assert_eq!(tree.root().data(), &201);
    ///
    /// let mut n10 = tree.node_mut(id10);
    /// *n10.root_mut().data_mut() += 100;
    /// assert_eq!(tree.root().data(), &301);
    /// ```
    pub fn root_mut(&'a mut self) -> NodeMut<'a, V, M, P, MO> {
        let ends = self.col.ends_mut().get();
        let root_ptr = ends.expect("Tree is not-empty, and hence, has a root");
        NodeMut::new(&mut self.col, root_ptr)
    }

    /// Returns a mutable reference to data of this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::new(0);
    ///
    /// let mut root = tree.root_mut();
    ///
    /// *root.data_mut() = 10;
    /// assert_eq!(root.data(), &10);
    ///
    /// let [idx_a] = root.push_children([1]);
    /// let mut node = tree.node_mut(idx_a);
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

    /// Swaps the data of this and the other node with the given `other_idx`.
    ///
    /// # Panics
    ///
    /// Panics if the `other_idx` is invalid.
    ///
    /// # See also
    ///
    /// See [`try_swap_nodes`] to swap two independent subtrees rooted at given node indices.
    ///
    /// [`try_swap_nodes`]: crate::Tree::try_swap_nodes
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
    /// //  ╱ ╲   ╱
    /// // 4   5 6
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let id1 = root.idx();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, id5] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// n3.push_child(6);
    ///
    /// // swap data of nodes to obtain
    ///
    /// //      2
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     5
    /// //  ╱ ╲   ╱
    /// // 4   3 6
    ///
    /// tree.node_mut(id4).swap_data_with(id4); // does nothing
    /// tree.node_mut(id2).swap_data_with(id1);
    /// tree.node_mut(id5).swap_data_with(id3);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [2, 1, 5, 4, 3, 6]);
    /// ```
    pub fn swap_data_with(&mut self, other_idx: NodeIdx<V>) {
        assert!(other_idx.0.is_valid_for(self.col), "{}", INVALID_IDX_ERROR);
        let a = self.node_ptr;
        let b = other_idx.0.node_ptr();
        Self::swap_data_of_nodes(a, b);
    }

    /// Swaps the data of this node with its parent's data, and returns true.
    ///
    /// Does nothing and returns false if this node is the root, and hence, has no parent.
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
    /// let mut tree = DynTree::new(1);
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// let id8 = tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // let's move 8 up to root one by one, swapping with its parents
    /// //      8
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     3
    /// //  ╱ ╲   ╱ ╲
    /// // 2   5 6   7
    /// // |     |  ╱ ╲
    /// // 4     9 10  11
    /// tree.node_mut(id8).swap_data_with_parent();
    /// tree.node_mut(id4).swap_data_with_parent();
    /// tree.node_mut(id2).swap_data_with_parent();
    ///
    /// let swapped = tree.root_mut().swap_data_with_parent();
    /// assert!(!swapped);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [8, 1, 3, 2, 5, 6, 7, 4, 9, 10, 11]);
    /// ```
    pub fn swap_data_with_parent(&mut self) -> bool {
        let a = self.node_ptr;
        let b = unsafe { &*a.ptr() }.prev().get();
        match b {
            Some(b) => {
                Self::swap_data_of_nodes(a, b);
                true
            }
            None => false,
        }
    }

    // growth - vertically

    /// Pushes a child node with the given `value`;
    /// returns the [`NodeIdx`] of the created node.
    ///
    /// If this node already has children, the new child is added to the end as the
    /// new right-most node among the children.
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
    /// // 7     8 9   10
    ///
    /// let mut tree = DynTree::<_>::new(0);
    ///
    /// let mut root = tree.root_mut();
    /// let id1 = root.push_child(1);
    /// let id2 = root.push_child(2);
    ///
    /// let mut n1 = tree.node_mut(id1);
    /// let id3 = n1.push_child(3);
    /// n1.push_child(4);
    ///
    /// tree.node_mut(id3).push_child(7);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let id5 = n2.push_child(5);
    /// let id6 = n2.push_child(6);
    ///
    /// tree.node_mut(id5).push_child(8);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id6).push_child(10);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [0, 1, 3, 7, 4, 2, 5, 8, 6, 9, 10]);
    /// ```
    pub fn push_child(&mut self, value: V::Item) -> NodeIdx<V> {
        let child_ptr = self.push_child_get_ptr(value);
        self.node_idx_for(child_ptr)
    }

    /// Pushes the given constant number of `values` as children of this node;
    /// returns the [`NodeIdx`] array of the created nodes.
    ///
    /// If this node already has children, the new children are added to the end as the
    /// new right-most nodes of the children.
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
    /// // 7     8 9   10
    ///
    /// let mut tree = DaryTree::<4, _>::new(0);
    ///
    /// let mut root = tree.root_mut();
    /// let [id1, id2] = root.push_children([1, 2]);
    ///
    /// let mut n1 = tree.node_mut(id1);
    /// let [id3, _] = n1.push_children([3, 4]);
    ///
    /// tree.node_mut(id3).push_child(7);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id5, id6] = n2.push_children([5, 6]);
    ///
    /// tree.node_mut(id5).push_child(8);
    /// tree.node_mut(id6).push_children([9, 10]);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [0, 1, 3, 7, 4, 2, 5, 8, 6, 9, 10]);
    /// ```
    pub fn push_children<const N: usize>(&mut self, values: [V::Item; N]) -> [NodeIdx<V>; N] {
        values.map(|child| {
            let child_ptr = self.push_child_get_ptr(child);
            self.node_idx_for(child_ptr)
        })
    }

    /// Pushes the given variable number of `values` as children of this node;
    /// returns the [`NodeIdx`] iterator of the created nodes.
    ///
    /// If this node already has children, the new children are added to the end as the
    /// new right-most nodes of the children.
    ///
    /// Importantly note that this method returns a **lazy** iterator.
    /// If the returned iterator is not consumed, the children will **not** be pushed.
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
    /// // 7     8 9   10
    ///
    /// let mut idx = vec![];
    ///
    /// let mut tree = DynTree::<_>::new(0);
    ///
    /// let mut root = tree.root_mut();
    /// idx.push(root.idx());
    /// idx.extend(root.extend_children(1..=2));
    ///
    /// let mut n1 = tree.node_mut(idx[1]);
    /// idx.extend(n1.extend_children([3, 4]));
    ///
    /// let mut n2 = tree.node_mut(idx[2]);
    /// idx.extend(n2.extend_children(5..=6));
    ///
    /// idx.push(tree.node_mut(idx[3]).push_child(7));
    ///
    /// idx.push(tree.node_mut(idx[5]).push_child(8));
    /// idx.extend(tree.node_mut(idx[6]).extend_children([9, 10]));
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [0, 1, 3, 7, 4, 2, 5, 8, 6, 9, 10]);
    /// ```
    pub fn extend_children<'b, I>(
        &'b mut self,
        values: I,
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P, MO>
    where
        I: IntoIterator<Item = V::Item>,
        I::IntoIter: 'b,
    {
        values.into_iter().map(|value| {
            let child_ptr = self.push_child_get_ptr(value);
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                child_ptr,
            ))
        })
    }

    /// Appends the entire `subtree` of another tree as a child of this node;
    /// and returns the [`NodeIdx`] of the created child node.
    ///
    /// In other words, the root of the subtree will be immediate child of this node,
    /// and the other nodes of the subtree will also be added with the same orientation
    /// relative to the subtree root.
    ///
    /// # Subtree Variants
    ///
    /// * **I.** Cloned / copied subtree
    ///   * A subtree cloned or copied from another tree.
    ///   * The source tree remains unchanged.
    ///   * Can be created by [`as_cloned_subtree`] and [`as_copied_subtree`] methods.
    ///   * ***O(n)***
    /// * **II.** Subtree moved out of another tree
    ///   * The subtree will be moved from the source tree to this tree.
    ///   * Can be created by [`into_subtree`] method.
    ///   * ***O(n)***
    /// * **III.** Another entire tree
    ///   * The other tree will be consumed and moved into this tree.
    ///   * ***O(1)***
    ///
    /// [`as_cloned_subtree`]: crate::NodeRef::as_cloned_subtree
    /// [`as_copied_subtree`]: crate::NodeRef::as_copied_subtree
    /// [`into_subtree`]: crate::NodeMut::into_subtree
    ///
    /// # Examples
    ///
    /// ## I. Append Subtree cloned-copied from another Tree
    ///
    /// Remains the source tree unchanged.
    ///
    /// Runs in ***O(n)*** time where n is the number of source nodes.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //     a          b
    /// // -----------------------
    /// //     0          5
    /// //    ╱ ╲        ╱ ╲
    /// //   1   2      6   7
    /// //  ╱ ╲         |  ╱ ╲
    /// // 3   4        8 9   10
    ///
    /// let mut a = DynTree::<_>::new(0);
    /// let [id1, _] = a.root_mut().push_children([1, 2]);
    /// let [id3, _] = a.node_mut(id1).push_children([3, 4]);
    ///
    /// let mut b = DaryTree::<4, _>::new(5);
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(id6).push_child(8);
    /// b.node_mut(id7).push_children([9, 10]);
    ///
    /// // clone b.subtree(n6) under a.n3
    /// // clone b.subtree(n7) under a.n0
    /// //        a
    /// // -----------------------
    /// //        0
    /// //       ╱|╲
    /// //      ╱ | ╲
    /// //     ╱  |  ╲
    /// //    ╱   |   ╲
    /// //   1    2    7
    /// //  ╱ ╲       ╱ ╲
    /// // 3   4     9   10
    /// // |
    /// // 6
    /// // |
    /// // 8
    ///
    /// let n6 = b.node(id6).as_cloned_subtree();
    /// a.node_mut(id3).push_child_tree(n6);
    ///
    /// let n7 = b.node(id7).as_copied_subtree();
    /// a.root_mut().push_child_tree(n7);
    ///
    /// // validate the trees
    ///
    /// let bfs_a: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_a, [0, 1, 2, 7, 3, 4, 9, 10, 6, 8]);
    ///
    /// let bfs_b: Vec<_> = b.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_b, [5, 6, 7, 8, 9, 10]); // unchanged
    /// ```
    ///
    /// ## II. Append Subtree moved out of another Tree
    ///
    /// The source subtree rooted at the given node will be removed from the source
    /// tree.
    ///
    /// Runs in ***O(n)*** time where n is the number of source nodes.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //     a          b
    /// // -----------------------
    /// //     0          5
    /// //    ╱ ╲        ╱ ╲
    /// //   1   2      6   7
    /// //  ╱ ╲         |  ╱ ╲
    /// // 3   4        8 9   10
    ///
    /// // into_lazy_reclaim: to keep the indices valid
    /// let mut a = DynTree::<_>::new(0).into_lazy_reclaim();
    /// let [id1, id2] = a.root_mut().push_children([1, 2]);
    /// a.node_mut(id1).push_children([3, 4]);
    ///
    /// // into_lazy_reclaim: to keep the indices valid
    /// let mut b = DaryTree::<4, _>::new(5).into_lazy_reclaim();
    /// let id5 = b.root().idx();
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(id6).push_child(8);
    /// b.node_mut(id7).push_children([9, 10]);
    ///
    /// // move b.subtree(n7) under a.n2
    /// // move a.subtree(n1) under b.n5
    /// //     a          b
    /// // -----------------------
    /// //     0          5
    /// //      ╲        ╱ ╲
    /// //       2      6   1
    /// //       |      |  ╱ ╲
    /// //       7      8 3   4
    /// //      ╱ ╲
    /// //     9   10
    ///
    /// let n7 = b.node_mut(id7).into_subtree();
    /// a.node_mut(id2).push_child_tree(n7);
    ///
    /// let n1 = a.node_mut(id1).into_subtree();
    /// b.node_mut(id5).push_child_tree(n1);
    ///
    /// // validate the trees
    ///
    /// let bfs_a: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_a, [0, 2, 7, 9, 10]);
    ///
    /// let bfs_b: Vec<_> = b.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_b, [5, 6, 1, 8, 3, 4]);
    /// ```
    ///
    /// ## III. Append Another Tree
    ///
    /// The source tree will be moved into the target tree.
    ///
    /// Runs in ***O(1)*** time.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //  tree    b      c
    /// // ----------------------
    /// //     0    4      2
    /// //    ╱     |     ╱ ╲
    /// //   1      7    5   6
    /// //  ╱            |  ╱ ╲
    /// // 3             8 9   10
    /// // ----------------------
    /// //      0
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     2
    /// //  ╱ ╲   ╱ ╲
    /// // 3   4 5   6
    /// //     | |  ╱ ╲
    /// //     7 8 9   10
    ///
    /// let mut tree = DynTree::<_>::new(0);
    /// let id0 = tree.root().idx();
    /// let id1 = tree.node_mut(id0).push_child(1);
    /// tree.node_mut(id1).push_child(3);
    ///
    /// let mut b = BinaryTree::<_>::new(4);
    /// b.root_mut().push_child(7);
    ///
    /// let mut c = DaryTree::<4, _>::new(2);
    /// let [id5, id6] = c.root_mut().push_children([5, 6]);
    /// c.node_mut(id5).push_child(8);
    /// c.node_mut(id6).push_children([9, 10]);
    ///
    /// // merge b & c into tree
    ///
    /// let id4 = tree.node_mut(id1).push_child_tree(b);
    /// let id2 = tree.node_mut(id0).push_child_tree(c);
    ///
    /// assert_eq!(tree.node(id2).data(), &2);
    /// assert_eq!(tree.node(id4).data(), &4);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    /// ```
    pub fn push_child_tree<Vs>(&mut self, subtree: impl SubTree<Vs>) -> NodeIdx<V>
    where
        Vs: TreeVariant<Item = V::Item>,
    {
        subtree.append_to_node_as_child(self, self.num_children())
    }

    /// Appends the entire `subtree` of this tree as a child of this node;
    /// and returns the [`NodeIdx`] of the created child node.
    ///
    /// In other words, the root of the subtree will be immediate child of this node,
    /// and the other nodes of the subtree will also be added with the same orientation
    /// relative to the subtree root.
    ///
    /// # Subtree Variants
    ///
    /// * **I.** Subtree moved out of this tree
    ///   * The subtree will be moved from its original to child of this node.
    ///   * Can be created by [`into_subtree_within`] method.
    ///   * **Panics** if the root of the subtree is an ancestor of this node.
    ///   * ***O(1)***
    /// * **II.** Cloned / copied subtree from this tree
    ///   * A subtree cloned or copied from another tree.
    ///   * The source tree remains unchanged.
    ///   * Can be created by [`as_cloned_subtree_within`] and [`as_copied_subtree_within`] methods.
    ///   * ***O(n)***
    ///
    /// # Panics
    ///
    /// Panics if the subtree is moved out of this tree created by [`into_subtree_within`] (**I.**) and
    /// the root of the subtree is an ancestor of this node.
    /// Notice that such a move would break structural properties of the tree.
    /// When we are not certain, we can test the relation using the the [`is_ancestor_of`] method.
    ///
    /// [`as_cloned_subtree_within`]: crate::NodeIdx::as_cloned_subtree_within
    /// [`as_copied_subtree_within`]: crate::NodeIdx::as_copied_subtree_within
    /// [`into_subtree_within`]: crate::NodeIdx::into_subtree_within
    /// [`is_ancestor_of`]: crate::NodeRef::is_ancestor_of
    ///
    /// # Examples
    ///
    /// ## I. Append Subtree moved from another position of this tree
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1                1              1
    /// //     ╱ ╲              ╱ ╲             |
    /// //    ╱   ╲            ╱   ╲            |
    /// //   2     3          2     3           2
    /// //  ╱ ╲   ╱ ╲   =>   ╱|╲   ╱ ╲    =>   ╱|╲
    /// // 4   5 6   7      4 5 8 6   7       4 5 8
    /// // |                                    |
    /// // 8                                    3
    /// //                                     ╱ ╲
    /// //                                    6   7
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, id5] = tree.node_mut(id2).push_children([4, 5]);
    /// let id8 = tree.node_mut(id4).push_child(8);
    /// tree.node_mut(id3).push_children([6, 7]);
    ///
    /// // move subtree rooted at n8 (single node) as a child of n2
    /// let st8 = id8.into_subtree_within();
    /// tree.node_mut(id2).push_child_tree_within(st8);
    ///
    /// // move subtree rooted at n3 (n3, n6 & n7) as a child of n5
    /// let st3 = id3.into_subtree_within();
    /// tree.node_mut(id5).push_child_tree_within(st3);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 4, 5, 8, 3, 6, 7]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 5, 3, 6, 7, 8]);
    /// ```
    ///
    /// ## II. Append Subtree cloned-copied from another position of this tree
    ///
    /// Remains the source tree unchanged.
    ///
    /// Runs in ***O(n)*** time where n is the number of source nodes.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1                1
    /// //     ╱ ╲              ╱ ╲
    /// //    ╱   ╲            ╱   ╲
    /// //   2     3          2     3
    /// //  ╱ ╲   ╱ ╲   =>   ╱ ╲   ╱|╲
    /// // 4   5 6   7      4   5 6 7 3
    /// //     |            |   |    ╱ ╲
    /// //     8            5   8   6   7
    /// //                  |
    /// //                  8
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, id5] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id5).push_child(8);
    /// tree.node_mut(id3).push_children([6, 7]);
    ///
    /// // clone subtree rooted at n5 as a child of n4
    /// let st5 = id5.as_cloned_subtree_within();
    /// tree.node_mut(id4).push_child_tree_within(st5);
    ///
    /// // copy subtree rooted at n3 (n3, n6 & n7) as a child of n3 (itself)
    /// let st3 = id3.as_cloned_subtree_within();
    /// tree.node_mut(id3).push_child_tree_within(st3);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 3, 5, 8, 6, 7, 8]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 5, 8, 5, 8, 3, 6, 7, 3, 6, 7]);
    /// ```
    pub fn push_child_tree_within(&mut self, subtree: impl SubTreeWithin<V>) -> NodeIdx<V> {
        subtree.append_to_node_within_as_child(self, self.num_children())
    }

    // growth - horizontally

    /// Pushes a sibling node with the given `value`:
    ///
    /// * as the immediate left-sibling of this node when `side` is [`Side::Left`],
    /// * as the immediate right-sibling of this node when `side` is [`Side::Right`],
    ///
    /// returns the [`NodeIdx`] of the created node.
    ///
    /// # Panics
    ///
    /// Panics if this node is the root; root node cannot have a sibling.
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
    /// //  ╱ ╲     ╲
    /// // 4   5     6
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6] = n3.push_children([6]);
    ///
    /// // grow horizontally to obtain
    /// //         1
    /// //        ╱ ╲
    /// //       ╱   ╲
    /// //      2     3
    /// //     ╱|╲    └────────
    /// //    ╱ | ╲          ╱ | ╲
    /// //   ╱ ╱ ╲ ╲        ╱  |  ╲
    /// //  ╱ ╱   ╲ ╲      ╱╲  |  ╱╲
    /// // 7 4    8  5    9 10 6 11 12
    ///
    /// let mut n4 = tree.node_mut(id4);
    /// n4.push_sibling(Side::Left, 7);
    /// n4.push_sibling(Side::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(id6);
    /// n6.push_sibling(Side::Left, 9);
    /// n6.push_sibling(Side::Left, 10);
    /// let id12 = n6.push_sibling(Side::Right, 12);
    /// let id11 = n6.push_sibling(Side::Right, 11);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// assert_eq!(tree.node(id12).data(), &12);
    /// assert_eq!(tree.node(id11).data(), &11);
    /// ```
    pub fn push_sibling(&mut self, side: Side, value: V::Item) -> NodeIdx<V> {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let position = match side {
            Side::Left => self.sibling_idx(),
            Side::Right => self.sibling_idx() + 1,
        };

        let ptr = Self::insert_sibling_get_ptr(self.col, value, parent_ptr, position);
        self.node_idx_for(ptr)
    }

    /// Pushes the given constant number of `values` as:
    ///
    /// * as the immediate left-siblings of this node when `side` is [`Side::Left`],
    /// * as the immediate right-siblings of this node when `side` is [`Side::Right`],
    ///
    /// returns the [`NodeIdx`] array of the created nodes.
    ///
    /// # Panics
    ///
    /// Panics if this node is the root; root node cannot have a sibling.
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
    /// //  ╱ ╲     ╲
    /// // 4   5     6
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6] = n3.push_children([6]);
    ///
    /// // grow horizontally to obtain
    /// //         1
    /// //        ╱ ╲
    /// //       ╱   ╲
    /// //      2     3
    /// //     ╱|╲    └────────
    /// //    ╱ | ╲          ╱ | ╲
    /// //   ╱ ╱ ╲ ╲        ╱  |  ╲
    /// //  ╱ ╱   ╲ ╲      ╱╲  |  ╱╲
    /// // 7 4    8  5    9 10 6 11 12
    ///
    /// let mut n4 = tree.node_mut(id4);
    /// n4.push_sibling(Side::Left, 7);
    /// n4.push_sibling(Side::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(id6);
    /// let [id9, id10] = n6.push_siblings(Side::Left, [9, 10]);
    /// let [id11, id12] = n6.push_siblings(Side::Right, [11, 12]);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// assert_eq!(tree.node(id9).data(), &9);
    /// assert_eq!(tree.node(id10).data(), &10);
    /// assert_eq!(tree.node(id11).data(), &11);
    /// assert_eq!(tree.node(id12).data(), &12);
    /// ```
    pub fn push_siblings<const N: usize>(
        &mut self,
        side: Side,
        values: [V::Item; N],
    ) -> [NodeIdx<V>; N] {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let mut position = match side {
            Side::Left => self.sibling_idx(),
            Side::Right => self.sibling_idx() + 1,
        };

        values.map(|sibling| {
            let sibling_ptr = Self::insert_sibling_get_ptr(self.col, sibling, parent_ptr, position);
            position += 1;
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                sibling_ptr,
            ))
        })
    }

    /// Pushes the given variable number of `values` as:
    ///
    /// * as the immediate left-siblings of this node when `side` is [`Side::Left`],
    /// * as the immediate right-siblings of this node when `side` is [`Side::Right`],
    ///
    /// returns the [`NodeIdx`] iterator of the created nodes.
    ///
    /// Importantly note that this method returns a **lazy** iterator.
    /// If the returned iterator is not consumed, the siblings will **not** be pushed.
    ///
    /// # Panics
    ///
    /// Panics if this node is the root; root node cannot have a sibling.
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
    /// //  ╱ ╲     ╲
    /// // 4   5     6
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6] = n3.push_children([6]);
    ///
    /// // grow horizontally to obtain
    /// //         1
    /// //        ╱ ╲
    /// //       ╱   ╲
    /// //      2     3
    /// //     ╱|╲    └────────
    /// //    ╱ | ╲          ╱ | ╲
    /// //   ╱ ╱ ╲ ╲        ╱  |  ╲
    /// //  ╱ ╱   ╲ ╲      ╱╲  |  ╱╲
    /// // 7 4    8  5    9 10 6 11 12
    ///
    /// let mut n4 = tree.node_mut(id4);
    /// n4.push_sibling(Side::Left, 7);
    /// n4.push_sibling(Side::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(id6);
    /// n6.extend_siblings(Side::Left, 9..=10).count();
    /// let idx: Vec<_> = n6.extend_siblings(Side::Right, 11..=12).collect();
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// assert_eq!(tree.node(idx[0]).data(), &11);
    /// assert_eq!(tree.node(idx[1]).data(), &12);
    /// ```
    pub fn extend_siblings<'b, I>(
        &'b mut self,
        side: Side,
        values: I,
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P, MO>
    where
        I: IntoIterator<Item = V::Item>,
        I::IntoIter: 'b,
    {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let mut position = match side {
            Side::Left => self.sibling_idx(),
            Side::Right => self.sibling_idx() + 1,
        };

        values.into_iter().map(move |sibling| {
            let sibling_ptr = Self::insert_sibling_get_ptr(self.col, sibling, parent_ptr, position);
            position += 1;
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                sibling_ptr,
            ))
        })
    }

    /// Appends the entire `subtree`:
    ///
    /// * as the immediate left-sibling of this node when `side` is [`Side::Left`],
    /// * as the immediate right-sibling of this node when `side` is [`Side::Right`],
    ///
    /// returns the [`NodeIdx`] of the sibling child node.
    ///
    /// In other words, the root of the subtree will be immediate sibling of this node,
    /// and the other nodes of the subtree will also be added with the same orientation
    /// relative to the subtree root.
    ///
    /// # Panics
    ///
    /// Panics if this node is the root; root node cannot have a sibling.
    ///
    /// # Subtree Variants
    ///
    /// * **I.** Cloned / copied subtree
    ///   * A subtree cloned or copied from another tree.
    ///   * The source tree remains unchanged.
    ///   * Can be created by [`as_cloned_subtree`] and [`as_copied_subtree`] methods.
    ///   * ***O(n)***
    /// * **II.** Subtree moved out of another tree
    ///   * The subtree will be moved from the source tree to this tree.
    ///   * Can be created by [`into_subtree`] method.
    ///   * ***O(n)***
    /// * **III.** Another entire tree
    ///   * The other tree will be consumed and moved into this tree.
    ///   * ***O(1)***
    ///
    /// [`as_cloned_subtree`]: crate::NodeRef::as_cloned_subtree
    /// [`as_copied_subtree`]: crate::NodeRef::as_copied_subtree
    /// [`into_subtree`]: crate::NodeMut::into_subtree
    ///
    /// # Examples
    ///
    /// ## I. Append Subtree cloned-copied from another Tree
    ///
    /// Remains the source tree unchanged.
    ///
    /// Runs in ***O(n)*** time where n is the number of source nodes.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //     a          b
    /// // -----------------------
    /// //     0          5
    /// //    ╱ ╲        ╱ ╲
    /// //   1   2      6   7
    /// //  ╱ ╲         |  ╱ ╲
    /// // 3   4        8 9   10
    ///
    /// let mut a = DynTree::<_>::new(0);
    /// let [id1, id2] = a.root_mut().push_children([1, 2]);
    /// let [_, id4] = a.node_mut(id1).push_children([3, 4]);
    ///
    /// let mut b = DaryTree::<4, _>::new(5);
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(id6).push_child(8);
    /// b.node_mut(id7).push_children([9, 10]);
    ///
    /// // clone b.subtree(n6) under a.n3
    /// // clone b.subtree(n7) under a.n0
    /// //        a
    /// // -----------------------
    /// //        0
    /// //       ╱|╲
    /// //      ╱ | ╲
    /// //     ╱  |  ╲
    /// //    ╱   |   ╲
    /// //   1    2    7
    /// //  ╱|╲       ╱ ╲
    /// // 3 6 4     9   10
    /// //   |
    /// //   8
    ///
    /// let n6 = b.node(id6).as_cloned_subtree();
    /// a.node_mut(id4).push_sibling_tree(Side::Left, n6);
    ///
    /// let n7 = b.node(id7).as_copied_subtree();
    /// a.node_mut(id2).push_sibling_tree(Side::Right, n7);
    ///
    /// // validate the trees
    ///
    /// let bfs_a: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_a, [0, 1, 2, 7, 3, 6, 4, 9, 10, 8]);
    ///
    /// let bfs_b: Vec<_> = b.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_b, [5, 6, 7, 8, 9, 10]); // unchanged
    /// ``````
    ///
    /// ## II. Append Subtree taken out of another Tree
    ///
    /// The source subtree rooted at the given node will be removed from the source
    /// tree.
    ///
    /// Runs in ***O(n)*** time where n is the number of source nodes.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //     a          b
    /// // -----------------------
    /// //     0          5
    /// //    ╱ ╲        ╱ ╲
    /// //   1   2      6   7
    /// //  ╱ ╲         |  ╱ ╲
    /// // 3   4        8 9   10
    ///
    /// // into_lazy_reclaim -> to keep indices valid
    /// let mut a = DynTree::<_>::new(0).into_lazy_reclaim();
    /// let [id1, id2] = a.root_mut().push_children([1, 2]);
    /// a.node_mut(id1).push_children([3, 4]);
    ///
    /// // into_lazy_reclaim -> to keep indices valid
    /// let mut b = DaryTree::<4, _>::new(5).into_lazy_reclaim();
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(id6).push_child(8);
    /// b.node_mut(id7).push_children([9, 10]);
    ///
    /// // move b.subtree(n7) under a.n2
    /// // move a.subtree(n1) under b.n5
    /// //     a          b
    /// // -----------------------
    /// //     0          5
    /// //    ╱ ╲        ╱ ╲
    /// //   7   2      6   1
    /// //  ╱ ╲         |  ╱ ╲
    /// // 9   10       8 3   4
    /// //
    /// //
    ///
    /// let n7 = b.node_mut(id7).into_subtree();
    /// a.node_mut(id2).push_sibling_tree(Side::Left, n7);
    ///
    /// let n1 = a.node_mut(id1).into_subtree();
    /// b.node_mut(id6).push_sibling_tree(Side::Right, n1);
    ///
    /// // validate the trees
    ///
    /// let bfs_a: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_a, [0, 7, 2, 9, 10]);
    ///
    /// let bfs_b: Vec<_> = b.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_b, [5, 6, 1, 8, 3, 4]);
    /// ```
    ///
    /// ## III. Append Another Tree
    ///
    /// The source tree will be moved into the target tree.
    ///
    /// Runs in ***O(1)*** time.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //  tree    b      c
    /// // ----------------------
    /// //     0    4      2
    /// //    ╱     |     ╱ ╲
    /// //   1      7    5   6
    /// //  ╱            |  ╱ ╲
    /// // 3             8 9   10
    /// // ----------------------
    /// //      0
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     2
    /// //  ╱ ╲   ╱ ╲
    /// // 4   3 5   6
    /// // |     |  ╱ ╲
    /// // 7     8 9   10
    ///
    /// let mut tree = DynTree::<_>::new(0);
    /// let id0 = tree.root().idx();
    /// let id1 = tree.node_mut(id0).push_child(1);
    /// let id3 = tree.node_mut(id1).push_child(3);
    ///
    /// let mut b = BinaryTree::<_>::new(4);
    /// b.root_mut().push_child(7);
    ///
    /// let mut c = DaryTree::<4, _>::new(2);
    /// let [id5, id6] = c.root_mut().push_children([5, 6]);
    /// c.node_mut(id5).push_child(8);
    /// c.node_mut(id6).push_children([9, 10]);
    ///
    /// // merge b & c into tree
    ///
    /// let id4 = tree.node_mut(id3).push_sibling_tree(Side::Left, b);
    /// let id2 = tree.node_mut(id1).push_sibling_tree(Side::Right, c);
    ///
    /// assert_eq!(tree.node(id2).data(), &2);
    /// assert_eq!(tree.node(id4).data(), &4);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 4, 3, 5, 6, 7, 8, 9, 10]);
    /// ```
    pub fn push_sibling_tree<Vs>(&mut self, side: Side, subtree: impl SubTree<Vs>) -> NodeIdx<V>
    where
        Vs: TreeVariant<Item = V::Item>,
    {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let position = match side {
            Side::Left => self.sibling_idx(),
            Side::Right => self.sibling_idx() + 1,
        };

        let mut parent = NodeMut::<V, M, P, MO>::new(self.col, parent_ptr);

        subtree.append_to_node_as_child(&mut parent, position)
    }

    /// Appends the entire `subtree`:
    ///
    /// * as the immediate left-sibling of this node when `side` is [`Side::Left`],
    /// * as the immediate right-sibling of this node when `side` is [`Side::Right`],
    ///
    /// returns the [`NodeIdx`] of the sibling child node.
    ///
    /// In other words, the root of the subtree will be immediate sibling of this node,
    /// and the other nodes of the subtree will also be added with the same orientation
    /// relative to the subtree root.
    ///
    /// # Subtree Variants
    ///
    /// * **I.** Subtree moved out of this tree
    ///   * The subtree will be moved from its original to child of this node.
    ///   * Can be created by [`into_subtree_within`] method.
    ///   * **Panics** if the root of the subtree is an ancestor of this node.
    ///   * ***O(1)***
    /// * **II.** Cloned / copied subtree from this tree
    ///   * A subtree cloned or copied from another tree.
    ///   * The source tree remains unchanged.
    ///   * Can be created by [`as_cloned_subtree_within`] and [`as_copied_subtree_within`] methods.
    ///   * ***O(n)***
    ///
    /// # Panics
    ///
    /// * Panics if this node is the root; root node cannot have a sibling.
    /// * Panics if the subtree is moved out of this tree created by [`into_subtree_within`] (**I.**) and
    ///   the root of the subtree is an ancestor of this node.
    ///   Notice that such a move would break structural properties of the tree.
    ///   When we are not certain, we can test the relation using the the [`is_ancestor_of`] method.
    ///
    /// [`as_cloned_subtree_within`]: crate::NodeIdx::as_cloned_subtree_within
    /// [`as_copied_subtree_within`]: crate::NodeIdx::as_copied_subtree_within
    /// [`into_subtree_within`]: crate::NodeIdx::into_subtree_within
    /// [`is_ancestor_of`]: crate::NodeRef::is_ancestor_of
    ///
    /// # Examples
    ///
    /// ## I. Append Subtree moved from another position of this tree
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1                1              1
    /// //     ╱ ╲              ╱ ╲             |
    /// //    ╱   ╲            ╱   ╲            |
    /// //   2     3          2     3           2
    /// //  ╱ ╲   ╱ ╲   =>   ╱|╲   ╱ ╲    =>   ╱|╲
    /// // 4   5 6   7      4 8 5 6   7       ╱ | ╲
    /// // |                                 ╱ ╱ ╲ ╲
    /// // 8                                4 3   8 5
    /// //                                   ╱ ╲
    /// //                                  6   7
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, id5] = tree.node_mut(id2).push_children([4, 5]);
    /// let id8 = tree.node_mut(id4).push_child(8);
    /// tree.node_mut(id3).push_children([6, 7]);
    ///
    /// // move subtree rooted at n8 (single node) as left sibling of n5
    /// let st8 = id8.into_subtree_within();
    /// tree.node_mut(id5)
    ///     .push_sibling_tree_within(Side::Left, st8);
    ///
    /// // move subtree rooted at n3 (n3, n6 & n7) as right sibling of n4
    /// let st3 = id3.into_subtree_within();
    /// tree.node_mut(id4)
    ///     .push_sibling_tree_within(Side::Right, st3);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 4, 3, 8, 5, 6, 7]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 3, 6, 7, 8, 5]);
    /// ```
    ///
    /// ## II. Append Subtree cloned-copied from another position of this tree
    ///
    /// Remains the source tree unchanged.
    ///
    /// Runs in ***O(n)*** time where n is the number of source nodes.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1                1
    /// //     ╱ ╲              ╱ ╲
    /// //    ╱   ╲            ╱   ╲
    /// //   2     3          2     3
    /// //  ╱ ╲   ╱ ╲   =>   ╱|╲   ╱|╲
    /// // 4   5 6   7      4 6 5 6 7 3
    /// //     |                |    ╱ ╲
    /// //     8                8   6   7
    /// //
    /// //
    ///
    /// let mut tree = DynTree::<_>::new(1);
    ///
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [_, id5] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id5).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    ///
    /// // clone subtree rooted at n6 as left sibling of n5
    /// let st6 = id6.as_cloned_subtree_within();
    /// tree.node_mut(id5)
    ///     .push_sibling_tree_within(Side::Left, st6);
    ///
    /// // copy subtree rooted at n3 (n3, n6 & n7) as right sibling of n7
    /// let st3 = id3.as_cloned_subtree_within();
    /// tree.node_mut(id7)
    ///     .push_sibling_tree_within(Side::Right, st3);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 6, 5, 6, 7, 3, 8, 6, 7]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 6, 5, 8, 3, 6, 7, 3, 6, 7]);
    /// ```
    pub fn push_sibling_tree_within(
        &mut self,
        side: Side,
        subtree: impl SubTreeWithin<V>,
    ) -> NodeIdx<V> {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let position = match side {
            Side::Left => self.sibling_idx(),
            Side::Right => self.sibling_idx() + 1,
        };

        let mut parent = NodeMut::<V, M, P, MO>::new(self.col, parent_ptr);

        subtree.append_to_node_within_as_child(&mut parent, position)
    }

    // move

    /// ***O(1)*** Inserts a node with the given `value` as the parent of this node;
    /// and returns the [`NodeIdx`] of the new parent node.
    ///
    /// As a result of this move:
    ///
    /// * this node and all its descendants will be down-shifted by one level in depth
    /// * this node will be the only child of the new parent node
    /// * this node's earlier parent will be the parent of the new parent node
    /// * if this node was the root, the new parent will now be the root
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //                       0
    /// //                       |
    /// //      1                1
    /// //     ╱ ╲              ╱ ╲
    /// //    ╱   ╲            ╱   ╲
    /// //   2     3     =>   6     7
    /// //        ╱ ╲         |     |
    /// //       4   5        2     3
    /// //                         ╱ ╲
    /// //                        4   5
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// n3.push_children([4, 5]);
    ///
    /// // push parent (insert a node vertically)
    ///
    /// let id0 = tree.root_mut().push_parent(0);
    /// let id6 = tree.node_mut(id2).push_parent(6);
    /// let id7 = tree.node_mut(id3).push_parent(7);
    ///
    /// // test inserted parent indices
    ///
    /// assert!(tree.node(id0).is_root());
    /// assert_eq!(tree.node(id6).data(), &6);
    /// assert_eq!(tree.node(id7).data(), &7);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 6, 7, 2, 3, 4, 5]);
    ///
    /// let dfs: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [0, 1, 6, 2, 7, 3, 4, 5]);
    /// ```
    pub fn push_parent(&mut self, value: V::Item) -> NodeIdx<V> {
        let parent_ptr = self.col.push(value);

        let child_ptr = self.node_ptr;
        let child = unsafe { &mut *child_ptr.ptr_mut() };

        let ancestor_ptr = child.prev().get();

        // down arrows
        match &ancestor_ptr {
            Some(ancestor_ptr) => {
                let ancestor = unsafe { &mut *ancestor_ptr.ptr_mut() };
                ancestor.next_mut().replace_with(child_ptr, parent_ptr);
            }
            None => {
                // this node was the root => parent will be the new root
                self.col.ends_mut().set_some(parent_ptr);
            }
        }

        let parent = unsafe { &mut *parent_ptr.ptr_mut() };
        parent.next_mut().push(child_ptr);

        // up arrows

        let child = unsafe { &mut *child_ptr.ptr_mut() };
        child.prev_mut().set_some(parent_ptr);

        parent.prev_mut().set(ancestor_ptr);

        self.node_idx_for(parent_ptr)
    }

    // shrink

    /// Removes this node and all of its descendants from the tree; and returns the
    /// data of this node.
    ///
    /// > **(!)** As a method that removes nodes from the tree, this method might result in invalidating indices that are
    /// > cached earlier in the [`Auto`] mode, but not in the [`Lazy`] mode. Please see the documentation of [MemoryPolicy]
    /// > for details of node index validity. Specifically, the examples in the "Lazy Memory Claim: Preventing Invalid Indices"
    /// > section presents a convenient way that allows us to make sure that the indices are valid.
    ///
    /// [`Auto`]: crate::Auto
    /// [`Lazy`]: crate::Lazy
    /// [`MemoryPolicy`]: crate::MemoryPolicy
    ///
    /// # See also
    ///
    /// Note that this method returns the data of this node, while the data of the descendants
    /// are dropped.
    ///
    /// If the data of the entire subtree is required, you may use [`into_walk`] method with
    /// the desired traversal to define the order of the returned iterator.
    ///
    /// [`into_walk`]: crate::NodeMut::into_walk
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let id8 = tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // prune n4 (removes 4 and 8)
    ///
    /// let data = tree.node_mut(id4).prune();
    /// assert_eq!(data, 4);
    ///
    /// let values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(values, [1, 2, 3, 5, 6, 7, 9, 10, 11]);
    ///
    /// assert_eq!(tree.get_node(id4), None);
    /// assert_eq!(tree.try_node(id8), Err(NodeIdxError::RemovedNode));
    ///
    /// // prune n3 (3, 6, 7, 9, 10, 11)
    ///
    /// let data = tree.node_mut(id3).prune();
    /// assert_eq!(data, 3);
    ///
    /// let values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(values, [1, 2, 5]);
    ///
    /// // prune the root: clear the entire (remaining) tree
    ///
    /// let data = tree.root_mut().prune();
    /// assert_eq!(data, 1);
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.get_root(), None);
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn prune(self) -> V::Item {
        // TODO: we have the option to choose any traversal here; they are all safe
        // with SelfRefCol. We can pick the fastest one after benchmarks.

        // # SAFETY: We use this shared reference to iterate over the pointers of the
        // descendent nodes. Using a mut reference to the collection, we will close
        // each of the descendent nodes that we visit. Closing a node corresponds to
        // taking its data out and emptying all of its previous and next links.
        // Close operation is lazy and does not invalidate the pointers that we the
        // shared reference to create.
        let iter = PostOrderIterPtr::<_, Val>::from((Default::default(), self.node_ptr));
        for ptr in iter {
            if ptr != self.node_ptr {
                self.col.close(ptr);
            }
        }

        let node = unsafe { &mut *self.node_ptr.ptr_mut() };
        if let Some(parent) = node.prev_mut().get() {
            let parent = unsafe { &mut *parent.ptr_mut() };
            let sibling_idx = parent
                .next_mut()
                .remove(unsafe { self.node_ptr.ptr() as usize });
            debug_assert!(sibling_idx.is_some());
        }

        let root_ptr = self.col.ends().get().expect("tree is not empty");
        if root_ptr == self.node_ptr {
            self.col.ends_mut().clear();
        }

        // # SAFETY: On the other hand, close_and_reclaim might trigger a reclaim
        // operation which moves around the nodes, invalidating other pointers;
        // however, only after 'self.node_ptr' is also closed.
        self.col.close_and_reclaim(self.node_ptr)
    }

    /// Removes this node and returns its data;
    /// and connects the children of this node to its parent.
    ///
    /// Therefore, unlike [`prune`], the resulting tree will contain only one less node.
    ///
    /// Assume that this node's parent had `n` children while this node is the i-th child.
    /// Further, assume that this node has `m` children.
    /// Then, the i-th element of the parent's children will be replaced with the m children.
    /// After the move, the parent will contain `n - 1 + m` children.
    ///
    /// [`prune`]: crate::NodeMut::prune
    ///
    /// > **(!)** As a method that removes nodes from the tree, this method might result in invalidating indices that are
    /// > cached earlier in the [`Auto`] mode, but not in the [`Lazy`] mode. Please see the documentation of [MemoryPolicy]
    /// > for details of node index validity. Specifically, the examples in the "Lazy Memory Claim: Preventing Invalid Indices"
    /// > section presents a convenient way that allows us to make sure that the indices are valid.
    ///
    /// [`Auto`]: crate::Auto
    /// [`Lazy`]: crate::Lazy
    /// [`MemoryPolicy`]: crate::MemoryPolicy
    ///
    /// # Panics
    ///
    /// Due to the fact that the tree can contain only one root, this move panics:
    ///
    /// * if this node is the root,
    /// * and it has more than one child.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1                    1                  1
    /// //     ╱ ╲                  ╱ ╲                ╱|╲
    /// //    ╱   ╲                ╱   ╲              ╱ | ╲
    /// //   2     3     (-n7)    2     3     (-n2)  4  5  3
    /// //  ╱ ╲   ╱ ╲     =>     ╱ ╲   ╱| ╲    =>    |    ╱| ╲
    /// // 4   5 6   7          4   5 6 10 11        8   6 10 11
    /// // |     |  ╱ ╲         |     |                  |
    /// // 8     9 10  11       8     9                  9
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // take out n7
    ///
    /// let d7 = tree.node_mut(id7).take_out();
    /// assert_eq!(d7, 7);
    /// assert_eq!(tree.try_node(id7), Err(NodeIdxError::RemovedNode));
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 10, 11, 8, 9]);
    ///
    /// // take out n2
    ///
    /// let d2 = tree.node_mut(id2).take_out();
    /// assert_eq!(d2, 2);
    /// assert_eq!(tree.get_node(id2), None);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 4, 5, 3, 8, 6, 10, 11, 9]);
    /// ```
    pub fn take_out(self) -> V::Item {
        assert!(
            !self.is_root() || self.num_children() == 1,
            "If taken out node is the root, it must have only one child which will be the new root."
        );

        let parent_ptr = self.parent_ptr();
        let sibling_idx = self.sibling_idx();

        for child_ptr in self.node().next().children_ptr() {
            let child = unsafe { &mut *child_ptr.ptr_mut() };
            child.prev_mut().set(parent_ptr);
        }

        match parent_ptr {
            None => {
                let first_child = self.node().next().children_ptr().next().cloned();
                self.col.ends_mut().set(first_child);
            }
            Some(parent_ptr) => {
                let parent = unsafe { &mut *parent_ptr.ptr_mut() };
                parent.next_mut().remove_at(sibling_idx);
                for child_ptr in self.node().next().children_ptr().rev().cloned() {
                    parent.next_mut().insert(sibling_idx, child_ptr);
                }
            }
        }

        self.col.close_and_reclaim(self.node_ptr)
    }

    /// Removes all children of this node together with the subtrees rooted at the children.
    /// This node remains in the tree while it becomes a leaf node if it was not already.
    ///
    /// Note that, `node.remove_children()` call is just a shorthand for:
    ///
    /// ```rust ignore
    /// for c in node.children_mut() {
    ///     _ = c.prune();
    /// }
    /// ```
    ///
    /// # Examples
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
    /// let mut tree = DynTree::new(1);
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // let's remove children of node 3
    /// tree.node_mut(id3).remove_children();
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 8]);
    /// ```
    pub fn remove_children(&mut self) {
        for c in self.children_mut() {
            _ = c.prune();
        }
    }

    // traversal

    /// Creates a custom mutable walk starting from this node such that:
    ///
    /// * the first element will be this node, say `n1`,
    /// * the second element will be node `n2 = next_node(n1)`,
    /// * the third element will be node `n3 = next_node(n2)`,
    /// * ...
    ///
    /// The iteration will terminate as soon as the `next_node` returns `None`.
    ///
    /// # Examples
    ///
    /// In the following example we create a custom iterator that walks down the tree as follows:
    ///
    /// * if the current node is not the last of its siblings, the next node will be its next sibling;
    /// * if the current node is the last of its siblings and if it has children, the next node will be its first child;
    /// * otherwise, the iteration will terminate.
    ///
    /// This walk strategy is implemented by the `next_node` function, and `custom_walk` is called with this strategy.
    ///
    /// ```rust
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //  ╱ ╲   ╱ ╲
    /// // 4   5 6   7
    ///
    /// fn next_node<'a, T>(node: DynNode<'a, T>) -> Option<DynNode<'a, T>> {
    ///     let sibling_idx = node.sibling_idx();
    ///     let is_last_sibling = sibling_idx == node.num_siblings() - 1;
    ///
    ///     match is_last_sibling {
    ///         true => node.get_child(0),
    ///         false => match node.parent() {
    ///             Some(parent) => {
    ///                 let child_idx = sibling_idx + 1;
    ///                 parent.get_child(child_idx)
    ///             }
    ///             None => None,
    ///         },
    ///     }
    /// }
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    /// tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id3).push_children([6, 7]);
    ///
    /// let mut root = tree.root_mut();
    /// for (i, x) in root.custom_walk_mut(next_node).enumerate() {
    ///     *x += (i + 1) * 100;
    /// }
    ///
    /// let values: Vec<_> = tree.root().custom_walk(next_node).copied().collect();
    /// assert_eq!(values, [101, 202, 303, 406, 507]);
    ///
    /// let all_values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(all_values, [101, 202, 303, 4, 5, 406, 507]);
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn custom_walk_mut<F>(&mut self, next_node: F) -> impl Iterator<Item = &'a mut V::Item>
    where
        F: Fn(Node<'a, V, M, P>) -> Option<Node<'a, V, M, P>>,
    {
        let iter_ptr = CustomWalkIterPtr::new(self.col(), Some(self.node_ptr()), next_node);
        iter_ptr.map(|ptr| {
            let node = unsafe { &mut *ptr.ptr_mut() };
            node.data_mut()
                .expect("node is returned by next_node and is active")
        })
    }

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
    /// let mut root = tree.root_mut();
    /// root.push_children([2, 3]);
    ///
    /// for c in 0..root.num_children() {
    ///     let mut node = root.get_child_mut(c).unwrap();
    ///
    ///     let val = *node.data();
    ///     let children = (0..val).map(|x| x + 1 + val);
    ///
    ///     let _ = node.extend_children(children).count();
    ///
    ///     for c in 0..node.num_children() {
    ///         let mut node = node.get_child_mut(c).unwrap();
    ///         node.push_child(*node.data() + 3);
    ///     }
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 3, 4, 4, 5, 6, 6, 7, 7, 8, 9]);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 3, 6, 4, 7, 3, 4, 7, 5, 8, 6, 9]);
    /// ```
    pub fn get_child_mut(&mut self, child_index: usize) -> Option<NodeMut<'_, V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
            .map(move |p| NodeMut::new(self.col, p))
    }

    /// Returns the mutable node of the `child-index`-th child of this node.
    ///
    /// See also [`into_child_mut`] for consuming traversal.
    ///
    /// [`into_child_mut`]: crate::NodeMut::into_child_mut
    ///
    /// # Panics
    ///
    /// Panics if the child index is out of bounds; i.e., `child_index >= self.num_children()`.
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
    /// let mut root = tree.root_mut();
    /// root.push_children([2, 3]);
    ///
    /// for c in 0..root.num_children() {
    ///     let mut node = root.child_mut(c);
    ///
    ///     let val = *node.data();
    ///     let children = (0..val).map(|x| x + 1 + val);
    ///
    ///     let _ = node.extend_children(children).count();
    ///
    ///     for c in 0..node.num_children() {
    ///         let mut node = node.child_mut(c);
    ///         node.push_child(*node.data() + 3);
    ///     }
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 3, 4, 4, 5, 6, 6, 7, 7, 8, 9]);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 3, 6, 4, 7, 3, 4, 7, 5, 8, 6, 9]);
    /// ```
    pub fn child_mut(&mut self, child_index: usize) -> NodeMut<'_, V, M, P> {
        self.get_child_mut(child_index)
            .expect("Given child_index is out of bounds; i.e., child_index >= self.num_children()")
    }

    /// Consumes this mutable node and returns the mutable node of the `child-index`-th child;
    /// returns None if the child index is out of bounds.
    ///
    /// See also [`get_child_mut`] for non-consuming access.
    ///
    /// [`get_child_mut`]: crate::NodeMut::get_child_mut
    ///
    /// # Examples
    ///
    /// The example below demonstrates one way to build a tree using `into_parent_mut` and `into_child_mut` methods.
    /// In this approach, we start from the mutable root node.
    /// Then, we convert one mutable node to another, always having only one mutable node.
    ///
    /// See also index returning growth methods for an alternative tree building approach, such as
    /// [`push_child`] and [`push_children`].
    ///
    /// [`push_child`]: crate::NodeMut::push_child
    /// [`push_children`]: crate::NodeMut::push_children
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
    /// let mut root = tree.root_mut();
    /// root.push_children(['a', 'b']);
    ///
    /// let mut a = root.into_child_mut(0).unwrap();
    /// a.push_children(['c', 'd', 'e']);
    ///
    /// let mut b = a.into_parent_mut().unwrap().into_child_mut(1).unwrap();
    /// b.push_children(['f', 'g']);
    ///
    /// let mut g = b.into_child_mut(1).unwrap();
    /// g.push_children(['h', 'i', 'j']);
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, ['r', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, ['r', 'a', 'c', 'd', 'e', 'b', 'f', 'g', 'h', 'i', 'j']);
    /// ```
    pub fn into_child_mut(self, child_index: usize) -> Option<NodeMut<'a, V, M, P>> {
        self.node()
            .next()
            .get_ptr(child_index)
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
    /// The `children_mut` iterator yields mutable nodes with the limited `NodeMutDown` orientation.
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
    /// let mut root = tree.root_mut();
    ///
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(10);
    /// tree.node_mut(id7).push_children([711, 712]);
    ///
    /// // push nodes 8 and 9 using children_mut of node 2
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// for mut child in n2.children_mut() {
    ///     let child_val = *child.data(); // 4 & 5
    ///     child.push_child(child_val + 4); // 8 & 9
    /// }
    ///
    /// // update values using children_mut of node 7
    ///
    /// let mut n7 = tree.node_mut(id7);
    /// for mut child in n7.children_mut() {
    ///     *child.data_mut() -= 700;
    /// }
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, [1, 2, 4, 8, 5, 9, 3, 6, 10, 7, 11, 12]);
    /// ```
    pub fn children_mut(
        &mut self,
    ) -> impl ExactSizeIterator<Item = NodeMut<'_, V, M, P, NodeMutDown>>
    + DoubleEndedIterator
    + use<'_, 'a, V, M, P, MO> {
        ChildrenMutIter::new(self.col, unsafe { self.node_ptr.ptr() })
    }

    /// Creates an iterator that yields mutable references to data of all nodes belonging to the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the generic [`Traverser`] parameter `T`.
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// See also [`walk`] and [`into_walk`] variants.
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
    /// [`walk`]: crate::NodeRef::walk
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
    /// let mut tree = DynTree::new(1);
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // walk over mutable references of nodes of any subtree
    /// // rooted at a selected node with different traversals
    ///
    /// let mut root = tree.root_mut();
    /// {
    ///     let mut bfs = root.walk_mut::<Bfs>();
    ///     assert_eq!(bfs.next(), Some(&mut 1));
    ///     assert_eq!(bfs.next(), Some(&mut 2)); // ...
    /// }
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// {
    ///     let mut dfs = n3.walk_mut::<Dfs>();
    ///     assert_eq!(dfs.next(), Some(&mut 3));
    ///     assert_eq!(dfs.next(), Some(&mut 6)); // ...
    /// }
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// {
    ///     let mut post_order = n2.walk_mut::<PostOrder>();
    ///     assert_eq!(post_order.next(), Some(&mut 8));
    ///     assert_eq!(post_order.next(), Some(&mut 4)); // ...
    /// }
    /// ```
    pub fn walk_mut<T>(&'a mut self) -> impl Iterator<Item = &'a mut V::Item>
    where
        T: Traverser<OverData>,
    {
        T::iter_mut_with_owned_storage::<V, M, P, MO>(self)
    }

    /// Creates an iterator that traverses all nodes belonging to the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`walk_mut`], this method does not require internal allocation.
    /// Furthermore, it allows to attach node depths or sibling indices to the yield values.
    /// Please see the examples below.
    ///
    /// [`walk_mut`]: crate::NodeMut::walk_mut
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
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
    /// let mut n7 = tree.node_mut(id7);
    /// for x in n7.walk_mut_with(&mut dfs) {
    ///     *x += 100;
    /// }
    /// let values: Vec<_> = tree.root().walk_with(&mut dfs).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 107, 110, 111]);
    ///
    /// let n3 = tree.node_mut(id3);
    /// let removed: Vec<_> = n3.into_walk_with(&mut dfs).collect();
    /// assert_eq!(removed, [3, 6, 9, 107, 110, 111]);
    ///
    /// let remaining: Vec<_> = tree.root().walk_with(&mut dfs).copied().collect();
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // create the traverser 'bfs' iterator
    /// // to walk over nodes rather than data
    ///
    /// let mut bfs = Bfs::default().over_nodes();
    /// // OR: Bfs::<OverNode>::new();
    ///
    /// let n7 = tree.node(id7);
    /// let mut iter = n7.walk_with(&mut bfs);
    /// let node = iter.next().unwrap();
    /// assert_eq!(node.num_children(), 2);
    /// assert_eq!(node.get_child(1).map(|x| *x.data()), Some(11));
    ///
    /// // or to additionally yield depth and/or sibling-idx
    ///
    /// let mut dfs = Dfs::default().with_depth().with_sibling_idx();
    /// // OR: Dfs::<OverDepthSiblingIdxData>::new()
    ///
    /// let n3 = tree.node(id3);
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
    pub fn walk_mut_with<T, O>(
        &'a mut self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        O: OverMut,
        T: Traverser<O>,
    {
        traverser.iter_mut(self)
    }

    /// Creates an iterator that yields owned (removed) data of all nodes belonging to the subtree rooted at this node.
    ///
    /// Note that once the returned iterator is dropped, regardless of whether it is completely used up or not,
    /// the subtree rooted at this node will be **removed** from the tree it belongs to.
    /// If this node is the root of the tree, the tree will be left empty.
    ///
    /// The order of the elements is determined by the generic [`Traverser`] parameter `T`.
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// See also [`walk`] and [`walk_mut`] for iterators over shared and mutable references, respectively.
    ///
    /// Note that tree traversing methods typically allocate a temporary data structure that is dropped once the
    /// iterator is dropped.
    /// In use cases where we repeatedly iterate using any of the **walk** methods over different nodes or different
    /// trees, we can avoid the allocation by creating the traverser only once and using [`walk_with`], [`walk_mut_with`]
    /// and [`into_walk_with`] methods instead.
    /// These methods additionally allow for iterating over nodes rather than data; and yielding node depths and sibling
    /// indices in addition to node data.
    ///
    /// > **(!)** As a method that removes nodes from the tree, this method might result in invalidating indices that are
    /// > cached earlier in the [`Auto`] mode, but not in the [`Lazy`] mode. Please see the documentation of [MemoryPolicy]
    /// > for details of node index validity. Specifically, the examples in the "Lazy Memory Claim: Preventing Invalid Indices"
    /// > section presents a convenient way that allows us to make sure that the indices are valid.
    ///
    /// [`Auto`]: crate::Auto
    /// [`Lazy`]: crate::Lazy
    /// [`MemoryPolicy`]: crate::MemoryPolicy
    ///
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
    /// [`walk`]: crate::NodeRef::walk
    /// [`walk_mut`]: crate::NodeMut::walk_mut
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
    /// // keep indices valid during removals
    /// let mut tree = DynTree::new(1).into_lazy_reclaim();
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // remove any subtree rooted at a selected node
    /// // from the tree, and collect the node values
    /// // in the order of different traversals
    ///
    /// let n4 = tree.node_mut(id4);
    /// let removed: Vec<_> = n4.into_walk::<PostOrder>().collect();
    /// assert_eq!(removed, [8, 4]);
    ///
    /// let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(remaining, [1, 2, 3, 5, 6, 7, 9, 10, 11]);
    ///
    /// let n3 = tree.node_mut(id3);
    /// let removed: Vec<_> = n3.into_walk::<Dfs>().collect();
    /// assert_eq!(removed, [3, 6, 9, 7, 10, 11]);
    ///
    /// let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(remaining, [1, 2, 5]);
    ///
    /// let root = tree.root_mut();
    /// let removed: Vec<_> = root.into_walk::<Bfs>().collect(); // empties the tree
    /// assert_eq!(removed, [1, 2, 5]);
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.get_root(), None);
    /// ```
    pub fn into_walk<T>(self) -> impl Iterator<Item = V::Item>
    where
        T: Traverser<OverData>,
    {
        T::into_iter_with_owned_storage::<V, M, P, MO>(self)
    }

    /// Creates an iterator that yields owned (removed) data of all nodes belonging to the subtree rooted at this node.
    ///
    /// Note that once the returned iterator is dropped, regardless of whether it is completely used up or not,
    /// the subtree rooted at this node will be **removed** from the tree it belongs to.
    /// If this node is the root of the tree, the tree will be left empty.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`into_walk`], this method does not require internal allocation.
    /// Furthermore, it allows to attach node depths or sibling indices to the yield values.
    /// Please see the examples below.
    ///
    /// [`into_walk`]: crate::NodeMut::into_walk
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
    ///
    /// > **(!)** As a method that removes nodes from the tree, this method might result in invalidating indices that are
    /// > cached earlier in the [`Auto`] mode, but not in the [`Lazy`] mode. Please see the documentation of [MemoryPolicy]
    /// > for details of node index validity. Specifically, the examples in the "Lazy Memory Claim: Preventing Invalid Indices"
    /// > section presents a convenient way that allows us to make sure that the indices are valid.
    ///
    /// [`Auto`]: crate::Auto
    /// [`Lazy`]: crate::Lazy
    /// [`MemoryPolicy`]: crate::MemoryPolicy
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
    /// // keep indices valid during removals
    /// let mut tree = DynTree::new(1).into_lazy_reclaim();
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
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
    /// let mut n7 = tree.node_mut(id7);
    /// for x in n7.walk_mut_with(&mut dfs) {
    ///     *x += 100;
    /// }
    /// let values: Vec<_> = tree.root().walk_with(&mut dfs).copied().collect();
    /// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 107, 110, 111]);
    ///
    /// let n3 = tree.node_mut(id3);
    /// let removed: Vec<_> = n3.into_walk_with(&mut dfs).collect();
    /// assert_eq!(removed, [3, 6, 9, 107, 110, 111]);
    ///
    /// let remaining: Vec<_> = tree.root().walk_with(&mut dfs).copied().collect();
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // create the traverser 'bfs' iterator
    /// // to walk over nodes rather than data
    ///
    /// let mut bfs = Bfs::default().over_nodes();
    /// // OR: Bfs::<OverNode>::new();
    ///
    /// let n7 = tree.node(id7);
    /// let mut iter = n7.walk_with(&mut bfs);
    /// let node = iter.next().unwrap();
    /// assert_eq!(node.num_children(), 2);
    /// assert_eq!(node.get_child(1).map(|x| *x.data()), Some(11));
    ///
    /// // or to additionally yield depth and/or sibling-idx
    ///
    /// let mut dfs = Dfs::default().with_depth().with_sibling_idx();
    /// // OR: Dfs::<OverDepthSiblingIdxData>::new()
    ///
    /// let n3 = tree.node(id3);
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
    pub fn into_walk_with<T, O>(
        self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        O: OverMut,
        T: Traverser<O>,
    {
        traverser.into_iter(self)
    }

    // traversal shorthands

    /// Returns an iterator of mutable references to data of leaves of the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // access the leaves in different orders that is determined by traversal
    ///
    /// let mut root = tree.root_mut();
    /// for (l, leaf) in root.leaves_mut::<Bfs>().enumerate() {
    ///     *leaf += 100 * l;
    /// }
    ///
    /// let bfs_leaves: Vec<_> = tree.root().leaves::<Bfs>().copied().collect();
    /// assert_eq!(bfs_leaves, [5, 108, 209, 310, 411]);
    ///
    /// // get the leaves from any node
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// for (l, leaf) in n3.leaves_mut::<PostOrder>().enumerate() {
    ///     *leaf -= 100 * l;
    /// }
    ///
    /// let n3 = tree.node(id3);
    /// let leaves: Vec<_> = n3.leaves::<PostOrder>().copied().collect();
    /// assert_eq!(leaves, [209, 210, 211]);
    /// ```
    pub fn leaves_mut<T>(&'a mut self) -> impl Iterator<Item = &'a mut V::Item>
    where
        T: Traverser<OverData>,
    {
        T::iter_ptr_with_owned_storage(self.node_ptr())
                .filter(|x: &NodePtr<V>| unsafe { &*x.ptr() }.next().is_empty())
                .map(|x: NodePtr<V>| {
                    <OverData as Over>::Enumeration::from_element_ptr_mut::<
                        'a,
                        V,
                        M,
                        P,
                        &'a mut V::Item,
                    >(self.col(), x)
                })
    }

    /// Creates an iterator of mutable references to leaves of the subtree rooted at this node.
    ///
    /// The order of the leaves is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`leaves_mut`], this method does not require internal allocation.
    /// Furthermore, it allows to attach node depths or sibling indices to the yield values.
    /// Please see the examples below.
    ///
    /// [`leaves_mut`]: crate::NodeMut::leaves_mut
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // create the traverser 'bfs' (or others) only once, use it many times
    /// // to walk over references, mutable references or removed values
    /// // without additional allocation
    ///
    /// let mut t = Bfs::default();
    ///
    /// for (l, leaf) in tree.root_mut().leaves_mut_with(&mut t).enumerate() {
    ///     *leaf += 100 * l;
    /// }
    ///
    /// let bfs_leaves: Vec<_> = tree.root().leaves_with(&mut t).copied().collect();
    /// assert_eq!(bfs_leaves, [5, 108, 209, 310, 411]);
    ///
    /// // get the leaves from any node
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// for (l, leaf) in n3.leaves_mut_with(&mut t).enumerate() {
    ///     *leaf -= 100 * l;
    /// }
    ///
    /// let n3 = tree.node(id3);
    /// let leaves: Vec<_> = n3.leaves_with(&mut t).copied().collect();
    /// assert_eq!(leaves, [209, 210, 211]);
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
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // create the traverser 'bfs' iterator which additionally
    /// // yields depths (or sibling_idx)
    ///
    /// let mut bfs = Traversal.bfs().with_depth();
    ///
    /// for (depth, x) in tree.root_mut().leaves_mut_with(&mut bfs) {
    ///     *x += 100 * depth;
    /// }
    ///
    /// let root = tree.root();
    /// let leaves: Vec<_> = root.leaves_with(&mut bfs).collect();
    /// assert_eq!(
    ///     leaves,
    ///     [(2, &205), (3, &308), (3, &309), (3, &310), (3, &311)]
    /// );
    /// ```
    pub fn leaves_mut_with<T, O>(
        &'a mut self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        O: OverMut,
        T: Traverser<O>,
    {
        T::iter_ptr_with_storage(self.node_ptr(), traverser.storage_mut())
            .filter(|x| {
                let ptr: &NodePtr<V> = O::Enumeration::node_data(x);
                unsafe { &*ptr.ptr() }.next().is_empty()
            })
            .map(|x| {
                O::Enumeration::from_element_ptr_mut::<'a, V, M, P, O::NodeItemMut<'a, V, M, P>>(
                    self.col(),
                    x,
                )
            })
    }

    /// Creates an iterator that yields owned (removed) data of all leaves of the subtree rooted at this node.
    ///
    /// Note that once the returned iterator is dropped, regardless of whether it is completely used up or not,
    /// the subtree rooted at this node will be **removed** from the tree it belongs to.
    /// If this node is the root of the tree, the tree will be left empty.
    ///
    /// The order of the elements is determined by the generic [`Traverser`] parameter `T`.
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// See also [`leaves`] and [`leaves_mut`] for iterators over shared and mutable references, respectively.
    ///
    /// Note that tree traversing methods typically allocate a temporary data structure that is dropped once the
    /// iterator is dropped.
    /// In use cases where we repeatedly iterate using any of the **leaves** methods over different nodes or different
    /// trees, we can avoid the allocation by creating the traverser only once and using [`leaves_with`], [`leaves_mut_with`]
    /// and [`into_leaves_with`] methods instead.
    /// These methods additionally allow for iterating over nodes rather than data; and yielding node depths and sibling
    /// indices in addition to node data.
    ///
    /// > **(!)** As a method that removes nodes from the tree, this method might result in invalidating indices that are
    /// > cached earlier in the [`Auto`] mode, but not in the [`Lazy`] mode. Please see the documentation of [MemoryPolicy]
    /// > for details of node index validity. Specifically, the examples in the "Lazy Memory Claim: Preventing Invalid Indices"
    /// > section presents a convenient way that allows us to make sure that the indices are valid.
    ///
    /// [`Auto`]: crate::Auto
    /// [`Lazy`]: crate::Lazy
    /// [`MemoryPolicy`]: crate::MemoryPolicy
    ///
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
    /// [`leaves`]: crate::NodeRef::leaves
    /// [`leaves_mut`]: crate::NodeMut::walk_mut
    /// [`leaves_with`]: crate::NodeRef::leaves_with
    /// [`leaves_mut_with`]: crate::NodeMut::leaves_mut_with
    /// [`into_leaves_with`]: crate::NodeMut::into_leaves_with
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
    /// let mut tree = DynTree::new(1);
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // keep indices valid during removals
    /// let mut tree = tree.into_lazy_reclaim();
    ///
    /// let n3 = tree.node_mut(id3);
    /// let leaves: Vec<_> = n3.into_leaves::<Dfs>().collect();
    /// assert_eq!(leaves, [9, 10, 11]);
    ///
    /// //      1
    /// //     ╱
    /// //    ╱
    /// //   2
    /// //  ╱ ╲
    /// // 4   5
    /// // |
    /// // 8
    /// let remaining: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    /// assert_eq!(remaining, [1, 2, 4, 8, 5]);
    ///
    /// let leaves: Vec<_> = tree.root_mut().into_leaves::<Dfs>().collect();
    /// assert_eq!(leaves, [8, 5]);
    ///
    /// assert!(tree.is_empty());
    /// ```
    pub fn into_leaves<T>(self) -> impl Iterator<Item = V::Item>
    where
        T: Traverser<OverData>,
    {
        let storage = T::Storage::<V>::default();
        T::into_iter_with_storage_filtered(self, storage, |x| {
            let ptr = <<OverData as Over>::Enumeration as Enumeration>::node_data(&x);
            unsafe { &*ptr.ptr() }.next().is_empty()
        })
    }

    /// Creates an iterator that yields owned (removed) data of all leaves of the subtree rooted at this node.
    ///
    /// Note that once the returned iterator is dropped, regardless of whether it is completely used up or not,
    /// the subtree rooted at this node will be **removed** from the tree it belongs to.
    /// If this node is the root of the tree, the tree will be left empty.
    ///
    /// The order of the elements is determined by the type of the `traverser` which implements [`Traverser`].
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for (pre-order) depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`into_leaves`], this method does not require internal allocation.
    /// Furthermore, it allows to attach node depths or sibling indices to the yield values.
    /// Please see the examples below.
    ///
    /// [`into_leaves`]: crate::NodeMut::into_leaves
    /// [`Bfs`]: crate::Bfs
    /// [`Dfs`]: crate::Dfs
    /// [`PostOrder`]: crate::PostOrder
    ///
    /// > **(!)** As a method that removes nodes from the tree, this method might result in invalidating indices that are
    /// > cached earlier in the [`Auto`] mode, but not in the [`Lazy`] mode. Please see the documentation of [MemoryPolicy]
    /// > for details of node index validity. Specifically, the examples in the "Lazy Memory Claim: Preventing Invalid Indices"
    /// > section presents a convenient way that allows us to make sure that the indices are valid.
    ///
    /// [`Auto`]: crate::Auto
    /// [`Lazy`]: crate::Lazy
    /// [`MemoryPolicy`]: crate::MemoryPolicy
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
    /// let mut tree = DynTree::new(1);
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // create the traverser 'dfs' only once, use it many times
    /// // to walk over references, mutable references or removed values
    /// // without additional allocation
    ///
    /// let mut dfs = Dfs::default();
    ///
    /// // keep indices valid during removals
    /// let mut tree = tree.into_lazy_reclaim();
    ///
    /// let n3 = tree.node_mut(id3);
    /// let leaves: Vec<_> = n3.into_leaves_with(&mut dfs).collect();
    /// assert_eq!(leaves, [9, 10, 11]);
    ///
    /// //      1
    /// //     ╱
    /// //    ╱
    /// //   2
    /// //  ╱ ╲
    /// // 4   5
    /// // |
    /// // 8
    /// let remaining: Vec<_> = tree.root().walk_with(&mut dfs).copied().collect();
    /// assert_eq!(remaining, [1, 2, 4, 8, 5]);
    ///
    /// let leaves: Vec<_> = tree.root_mut().into_leaves_with(&mut dfs).collect();
    /// assert_eq!(leaves, [8, 5]);
    ///
    /// assert!(tree.is_empty());
    /// ```
    pub fn into_leaves_with<T, O>(
        self,
        traverser: &'a mut T,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        O: OverMut,
        T: Traverser<O>,
    {
        T::into_iter_with_storage_filtered(self, traverser.storage_mut(), |x| {
            let ptr = <O::Enumeration as Enumeration>::node_data(x);
            unsafe { &*ptr.ptr() }.next().is_empty()
        })
    }

    // recursive

    /// Recursively sets the data of all nodes belonging to the subtree rooted at this node using the `compute_data`
    /// function.
    ///
    /// This method provides an expressive way to update the values of a tree where value of a node is a function of
    /// its prior value and values of its children. Since the values of its children subsequently depend on their own
    /// children, it immediately follows that the value of the node depends on values of all of its descendants that
    /// must be computed to be able to compute the node's value.
    ///
    /// The `compute_data` function takes two arguments:
    ///
    /// * current value (data) of this node, and
    /// * slice of values of children of this node that are computed recursively using `compute_data` (*);
    ///
    /// and then, computes the new value of this node.
    ///
    /// The method is named *recursive* (*) due to the fact that,
    ///
    /// * before computing the value of this node;
    /// * values of all of its children are also computed and set using the `compute_data` function.
    ///
    /// *Note that this method does **not** actually make recursive method calls. Instead, it internally uses the [`PostOrder`]
    /// traverser which ensures that all required values are computed before they are used for another computation. This
    /// is a guard against potential stack overflow issues, and hence, can be used for trees of arbitrary depth.*
    ///
    /// [`PostOrder`]: crate::PostOrder
    ///
    /// # Examples
    ///
    /// In the following example, we set the value of every node to the sum of values of all its descendants.
    ///
    /// While building the tree, we set only the values of the leaves.
    /// We initially set values of all other nodes to zero as a placeholder.
    /// Then, we call `recursive_set` to compute them.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<_>::new(0);
    /// let [id1, id2] = tree.root_mut().push_children([0, 0]);
    /// tree.node_mut(id1).push_children([1, 3]);
    /// tree.node_mut(id2).push_children([7, 2, 4]);
    /// //      0
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   0     0
    /// //  ╱ ╲   ╱|╲
    /// // 1   3 7 2 4
    ///
    /// tree.root_mut()
    ///     .recursive_set(
    ///         |current_value, children_values| match children_values.is_empty() {
    ///             true => *current_value, // is a leaf
    ///             false => children_values.iter().copied().sum(),
    ///         },
    ///     );
    /// //      17
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   4    13
    /// //  ╱ ╲   ╱|╲
    /// // 1   3 7 2 4
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [17, 4, 13, 1, 3, 7, 2, 4]);
    /// ```
    ///
    /// The following is a similar example where leaf nodes represent deterministic outcomes of
    /// a process.
    /// The root represents the current state.
    /// The remaining nodes represent intermediate states that we can reach from its parent with
    /// the given `probability`.
    /// Our task is to compute `expected_value` of each state.
    ///
    /// Since we know the value of the leaves with certainty, we set them while constructing the
    /// tree. Then, we call `recursive_set` to compute the expected value of every other node.
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// #[derive(Clone)]
    /// struct State {
    ///     /// Probability of reaching this state from its parent.
    ///     probability: f64,
    ///     /// Expected value of the state; i.e., average of values of all leaves weighted by
    ///     /// the probability of being reached from this state.
    ///     expected_value: f64,
    /// }
    ///
    /// fn state(probability: f64, expected_value: f64) -> State {
    ///     State {
    ///         probability,
    ///         expected_value,
    ///     }
    /// }
    ///
    /// //         (1.0, ???)
    /// //         ╱         ╲
    /// //        ╱           ╲
    /// //       ╱             ╲
    /// //      ╱               ╲
    /// //  (.3, ???)        (.7, ???)
    /// //   ╱     ╲          |    ╲
    /// //  ╱       ╲         |     ╲
    /// // (.2, 9) (.8, 2) (.9, 5) (.1, 4)
    ///
    /// let mut tree = DynTree::<_>::new(state(1.0, 0.0));
    ///
    /// let [id1, id2] = tree
    ///     .root_mut()
    ///     .push_children([state(0.3, 0.0), state(0.7, 0.0)]);
    /// tree.node_mut(id1)
    ///     .push_children([state(0.2, 9.0), state(0.8, 2.0)]);
    /// tree.node_mut(id2)
    ///     .push_children([state(0.9, 5.0), state(0.1, 4.0)]);
    ///
    /// tree.root_mut()
    ///     .recursive_set(
    ///         |current_value, children_values| match children_values.is_empty() {
    ///             true => current_value.clone(), // is a leaf, we know expected value
    ///             false => {
    ///                 let expected_value = children_values
    ///                     .iter()
    ///                     .fold(0.0, |a, x| a + x.probability * x.expected_value);
    ///                 state(current_value.probability, expected_value)
    ///             }
    ///         },
    ///     );
    /// //         (1.0, 4.45)
    /// //         ╱         ╲
    /// //        ╱           ╲
    /// //       ╱             ╲
    /// //      ╱               ╲
    /// //   (.3, 3.4)      (.7, 4.9)
    /// //   ╱     ╲          |    ╲
    /// //  ╱       ╲         |     ╲
    /// // (.2, 9) (.8, 2) (.9, 5) (.1, 4)
    ///
    /// let equals = |a: f64, b: f64| (a - b).abs() < 1e-5;
    ///
    /// assert!(equals(tree.root().data().expected_value, 4.45));
    /// assert!(equals(tree.node(id1).data().expected_value, 3.40));
    /// assert!(equals(tree.node(id2).data().expected_value, 4.90));
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn recursive_set(&mut self, compute_data: impl Fn(&V::Item, &[&V::Item]) -> V::Item) {
        let iter = PostOrder::<OverPtr>::iter_ptr_with_owned_storage(self.node_ptr);
        let mut children_data = Vec::<&V::Item>::new();

        for ptr in iter {
            let node = unsafe { &mut *ptr.ptr_mut() };
            let node_data = node.data().expect("is not closed");

            for child_ptr in node.next().children_ptr() {
                let data = unsafe { &*child_ptr.ptr() }.data().expect("is not closed");
                children_data.push(data);
            }

            let new_data = compute_data(node_data, &children_data);

            *node.data_mut().expect("is not closed") = new_data;

            children_data.clear();
        }
    }

    // subtree

    /// Creates a subtree view including this node as the root and all of its descendants with their orientation relative
    /// to this node.
    ///
    /// Consuming the created subtree in methods such as [`push_child_tree`] or [`push_sibling_tree`] will remove the
    /// subtree from this tree and move it to the target tree.
    /// Please see **Append Subtree taken out of another Tree** section of the examples of these methods.
    ///
    /// Otherwise, it has no impact on the tree.
    ///
    /// [`push_child_tree`]: crate::NodeMut::push_child_tree
    /// [`push_sibling_tree`]: crate::NodeMut::push_sibling_tree
    pub fn into_subtree(self) -> MovedSubTree<'a, V, M, P, MO> {
        MovedSubTree::new(self)
    }

    /// Removes the subtree rooted at this node from its tree; moves it into a new tree where this node is the root.
    ///
    /// See also [`clone_as_tree`] in order to keep the original tree unchanged.
    ///
    /// [`clone_as_tree`]: crate::NodeRef::clone_as_tree
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
    /// let mut tree = DynTree::new(1).into_lazy_reclaim(); // ensure index validity
    /// let [id2, id3] = tree.root_mut().push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// // let's move subtree rooted at n2 into new tree: tree2
    /// //   2
    /// //  ╱ ╲
    /// // 4   5
    /// // |
    /// // 8
    /// let tree2: DynTree<_> = tree.node_mut(id2).into_new_tree();
    /// let bfs: Vec<_> = tree2.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [2, 4, 5, 8]);
    ///
    /// // let's move subtree rooted at n7 into new tree: tree7
    /// // this time, the target tree is a BinaryTree
    /// //   7
    /// //  ╱ ╲
    /// // 10  11
    /// let tree7: BinaryTree<_> = tree.node_mut(id7).into_new_tree();
    /// let bfs: Vec<_> = tree7.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [7, 10, 11]);
    ///
    /// // these subtrees are removed from the original tree
    /// // 1
    /// //  ╲
    /// //   ╲
    /// //    3
    /// //   ╱
    /// //  6
    /// //  |
    /// //  9
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 3, 6, 9]);
    /// ```
    pub fn into_new_tree<V2>(self) -> Tree<V2, Auto, P>
    where
        V2: TreeVariant<Item = V::Item>,
        P::PinnedVec<V2>: Default,
    {
        self.into_subtree().into_new_tree()
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

    pub(crate) fn push_child_get_ptr(&mut self, value: V::Item) -> NodePtr<V> {
        let parent_ptr = self.node_ptr;

        let child_ptr = self.col.push(value);

        let child = self.col.node_mut(child_ptr);
        child.prev_mut().set_some(parent_ptr);

        let parent = self.col.node_mut(parent_ptr);
        parent.next_mut().push(child_ptr);

        child_ptr
    }

    fn insert_sibling_get_ptr(
        col: &mut Col<V, M, P>,
        value: V::Item,
        parent_ptr: NodePtr<V>,
        position: usize,
    ) -> NodePtr<V> {
        let sibling_ptr = col.push(value);

        let child = col.node_mut(sibling_ptr);
        child.prev_mut().set_some(parent_ptr);

        let parent = col.node_mut(parent_ptr);
        parent.next_mut().insert(position, sibling_ptr);

        sibling_ptr
    }

    pub(crate) fn into_inner(self) -> (&'a mut Col<V, M, P>, NodePtr<V>) {
        (self.col, self.node_ptr)
    }

    pub(crate) fn parent_ptr(&self) -> Option<NodePtr<V>> {
        self.node().prev().get()
    }

    /// Returns the pointer to the root; None if empty.
    pub(crate) fn root_ptr(&self) -> Option<NodePtr<V>> {
        self.col.ends().get()
    }

    fn node_idx_for(&self, ptr: NodePtr<V>) -> NodeIdx<V> {
        NodeIdx(orx_selfref_col::NodeIdx::new(self.col.memory_state(), ptr))
    }

    /// Tries to append the `subtree` as the `child_position`-th child of this node.
    ///
    /// This operation might only fail if the there is an increasing jump in depths that is
    /// greater than one.
    /// The method returns the (depth, succeeding_depth) pair as the error when this error
    /// is observed.
    ///
    /// # Panics
    ///
    /// Panics if the `subtree` is an empty iterator. It must contain at least one child node.
    #[allow(clippy::unwrap_in_result)]
    pub(crate) fn try_append_subtree_as_child(
        &mut self,
        subtree: impl IntoIterator<Item = (usize, V::Item)>,
        child_position: usize,
    ) -> Result<NodeIdx<V>, (usize, usize)> {
        let mut iter = subtree.into_iter();
        let (mut current_depth, value) = iter.next().expect("tree is not empty");

        let idx = match child_position == self.num_children() {
            true => self.push_child(value),
            false => {
                let ptr =
                    Self::insert_sibling_get_ptr(self.col, value, self.node_ptr, child_position);
                self.node_idx_for(ptr)
            }
        };

        let position = child_position;
        let mut dst = self.get_child_mut(position).expect("child exists");

        for (depth, value) in iter {
            match depth > current_depth {
                true => {
                    if depth > current_depth + 1 {
                        return Err((current_depth, depth));
                    }
                }
                false => {
                    let num_parent_moves = current_depth - depth + 1;
                    for _ in 0..num_parent_moves {
                        dst = dst.into_parent_mut().expect("in bounds");
                    }
                }
            }
            let position = dst.num_children();
            dst.push_child(value);
            dst = dst.into_child_mut(position).expect("child exists");
            current_depth = depth;
        }

        Ok(idx)
    }

    /// Appends the `subtree` as the `child_position`-th child of this node.
    ///
    /// # Panics
    ///
    /// It is only safe to call this method where the `subtree` represents a valid depth-first sequence.
    /// Note that any sequence created by [`Dfs`] iterator using the [`OverDepthData`] is always valid, and hence, the conversion cannot fail.
    ///
    /// Please see [`DepthFirstSequence`] for validity conditions.
    ///
    /// [`DepthFirstSequence`]: crate::DepthFirstSequence
    /// [`Dfs`]: crate::Dfs
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    pub(crate) fn append_subtree_as_child(
        &mut self,
        subtree: impl IntoIterator<Item = (usize, V::Item)>,
        child_position: usize,
    ) -> NodeIdx<V> {
        self.try_append_subtree_as_child(subtree, child_position)
            .expect("Since the depth first sequence is created by internal Dfs walk methods, sequence to subtree conversion cannot fail")
    }

    /// Swaps the data of the two valid nodes a and b, if they are different nodes.
    /// Does nothing if a == b.
    fn swap_data_of_nodes(a: NodePtr<V>, b: NodePtr<V>) {
        if a != b {
            let a = unsafe { &mut *a.ptr_mut() };
            let b = unsafe { &mut *b.ptr_mut() };
            core::mem::swap(
                a.data_mut().expect("valid idx"),
                b.data_mut().expect("valid idx"),
            );
        }
    }

    pub(crate) unsafe fn clone_node_mut(&mut self) -> Self {
        let node_ptr = self.node_ptr;
        let col = self.col as *mut Col<V, M, P>;
        Self {
            col: unsafe { &mut *col },
            node_ptr,
            phantom: PhantomData,
        }
    }
}

impl<'a, V, M, P> NodeMut<'a, V, M, P, NodeMutUpAndDown>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
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
    /// See also index returning growth methods for an alternative tree building approach, such as
    /// [`push_child`] and [`push_children`].
    ///
    /// [`push_child`]: crate::NodeMut::push_child
    /// [`push_children`]: crate::NodeMut::push_children
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
    /// let mut root = tree.root_mut();
    /// let [id_a, id_b] = root.push_children(['a', 'b']);
    ///
    /// let mut a = tree.node_mut(id_a);
    /// a.push_children(['c', 'd', 'e']);
    ///
    /// let mut b = tree.node_mut(id_b);
    /// let [_, id_g] = b.push_children(['f', 'g']);
    ///
    /// let mut g = tree.node_mut(id_g);
    /// let mut b = g.parent_mut().unwrap();
    /// let mut root = b.parent_mut().unwrap();
    ///
    /// *root.data_mut() = 'x';
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, ['x', 'a', 'b', 'c', 'd', 'e', 'f', 'g']);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, ['x', 'a', 'c', 'd', 'e', 'b', 'f', 'g']);
    /// ```
    pub fn parent_mut(&mut self) -> Option<NodeMut<'_, V, M, P>> {
        self.node().prev().get().map(|p| NodeMut::new(self.col, p))
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
    /// See also index returning growth methods for an alternative tree building approach, such as
    /// [`push_child`] and [`push_children`].
    ///
    /// [`push_child`]: crate::NodeMut::push_child
    /// [`push_children`]: crate::NodeMut::push_children
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
    /// let mut root = tree.root_mut();
    /// root.push_children(['a', 'b']);
    ///
    /// let mut a = root.into_child_mut(0).unwrap();
    /// a.push_children(['c', 'd', 'e']);
    ///
    /// let mut b = a.into_parent_mut().unwrap().into_child_mut(1).unwrap();
    /// b.push_children(['f', 'g']);
    ///
    /// let mut g = b.into_child_mut(1).unwrap();
    /// g.push_children(['h', 'i', 'j']);
    ///
    /// // validate the tree
    ///
    /// let root = tree.root();
    ///
    /// let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, ['r', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j']);
    ///
    /// let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    /// assert_eq!(dfs, ['r', 'a', 'c', 'd', 'e', 'b', 'f', 'g', 'h', 'i', 'j']);
    /// ```
    pub fn into_parent_mut(self) -> Option<NodeMut<'a, V, M, P>> {
        self.node().prev().get().map(|p| NodeMut::new(self.col, p))
    }
}
