use crate::{
    helpers::{Col, N},
    iter::ChildrenMutIter,
    memory::{Auto, MemoryPolicy},
    node_ref::NodeRefCore,
    pinned_storage::{PinnedStorage, SplitRecursive},
    subtrees::NodeMutAsSubTree,
    traversal::{
        enumerations::Val,
        over_mut::{OverItemInto, OverItemMut},
        post_order::iter_ptr::PostOrderIterPtr,
        OverData, OverMut,
    },
    tree_node_idx::INVALID_IDX_ERROR,
    tree_variant::RefsChildren,
    NodeIdx, NodeRef, SubTree, Traverser, TreeVariant,
};
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
    /// let mut root = tree.root_mut();
    ///
    /// *root.data_mut() = 10;
    /// assert_eq!(root.data(), &10);
    ///
    /// let [idx_a] = root.push_children([1]);
    /// let mut node = tree.node_mut(&idx_a);
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
    /// See [`swap_nodes`] to swap two independent subtrees rooted at given node indices.
    ///
    /// [`swap_nodes`]: crate::Tree::swap_nodes
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
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let id1 = root.idx();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, id5] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
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
    /// tree.node_mut(&id4).swap_data_with(&id4); // does nothing
    /// tree.node_mut(&id2).swap_data_with(&id1);
    /// tree.node_mut(&id5).swap_data_with(&id3);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [2, 1, 5, 4, 3, 6]);
    /// ```
    pub fn swap_data_with(&mut self, other_idx: &NodeIdx<V>) {
        assert!(other_idx.0.is_valid_for(self.col), "{}", INVALID_IDX_ERROR);
        let a = self.node_ptr.clone();
        let b = other_idx.0.node_ptr();

        if a != b {
            let a = unsafe { &mut *a.ptr_mut() };
            let b = unsafe { &mut *b.ptr_mut() };
            core::mem::swap(
                a.data_mut().expect("valid idx"),
                b.data_mut().expect("valid idx"),
            );
        }
    }

    // growth - vertically

    /// Pushes a child node with the given `value`;
    /// returns the [`NodeIdx`] of the created node.
    ///
    /// If this node already has children, the new child is added to the end as the
    /// new right-most node among the children.
    ///
    /// # Panics
    ///
    /// Panics if the tree is of a variant with fixed children capacity,
    /// such as 2 for [`BinaryTree`] or `D` for [`DaryTree`] in general,
    /// and if this hard capacity is violated with the new child.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
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
    /// let mut n1 = tree.node_mut(&id1);
    /// let id3 = n1.push_child(3);
    /// n1.push_child(4);
    ///
    /// tree.node_mut(&id3).push_child(7);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let id5 = n2.push_child(5);
    /// let id6 = n2.push_child(6);
    ///
    /// tree.node_mut(&id5).push_child(8);
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id6).push_child(10);
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
    pub fn push_child(&mut self, value: V::Item) -> NodeIdx<V> {
        let child_ptr = self.push_child_get_ptr(value);
        self.node_idx_for(&child_ptr)
    }

    /// Pushes the given constant number of `values` as children of this node;
    /// returns the [`NodeIdx`] array of the created nodes.
    ///
    /// If this node already has children, the new children are added to the end as the
    /// new right-most nodes of the children.
    ///
    /// # Panics
    ///
    /// Panics if the tree is of a variant with fixed children capacity,
    /// such as 2 for [`BinaryTree`] or `D` for [`DaryTree`] in general,
    /// and if this hard capacity is violated with the new child.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
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
    /// let mut n1 = tree.node_mut(&id1);
    /// let [id3, _] = n1.push_children([3, 4]);
    ///
    /// tree.node_mut(&id3).push_child(7);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id5, id6] = n2.push_children([5, 6]);
    ///
    /// tree.node_mut(&id5).push_child(8);
    /// tree.node_mut(&id6).push_children([9, 10]);
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
    pub fn push_children<const N: usize>(&mut self, values: [V::Item; N]) -> [NodeIdx<V>; N] {
        values.map(|child| {
            let child_ptr = self.push_child_get_ptr(child);
            self.node_idx_for(&child_ptr)
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
    /// # Panics
    ///
    /// Panics if the tree is of a variant with fixed children capacity,
    /// such as 2 for [`BinaryTree`] or `D` for [`DaryTree`] in general,
    /// and if this hard capacity is violated with the new child.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
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
    /// let mut n1 = tree.node_mut(&idx[1]);
    /// idx.extend(n1.extend_children([3, 4]));
    ///
    /// let mut n2 = tree.node_mut(&idx[2]);
    /// idx.extend(n2.extend_children(5..=6));
    ///
    /// idx.push(tree.node_mut(&idx[3]).push_child(7));
    ///
    /// idx.push(tree.node_mut(&idx[5]).push_child(8));
    /// idx.extend(tree.node_mut(&idx[6]).extend_children([9, 10]));
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
                &child_ptr,
            ))
        })
    }

    /// Appends the entire `subtree` as a child of this node;
    /// and returns the [`NodeIdx`] of the created child node.
    ///
    /// In other words, the root of the subtree will be immediate sibling of this node,
    /// and the other nodes of the subtree will also be added with the same orientation
    /// relative to the subtree root.
    ///
    /// # Panics
    ///
    /// Panics if the tree is of a variant with fixed children capacity,
    /// such as 2 for [`BinaryTree`] or `D` for [`DaryTree`] in general,
    /// and if this hard capacity is violated with the new child.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
    ///
    /// # Examples
    ///
    /// ## Append Subtree cloned-copied from another Tree
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
    /// let [id3, _] = a.node_mut(&id1).push_children([3, 4]);
    ///
    /// let mut b = DaryTree::<4, _>::new(5);
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(&id6).push_child(8);
    /// b.node_mut(&id7).push_children([9, 10]);
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
    /// let n6 = b.node(&id6).as_cloned_subtree();
    /// a.node_mut(&id3).append_child_tree(n6);
    ///
    /// let n7 = b.node(&id7).as_copied_subtree();
    /// a.root_mut().append_child_tree(n7);
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
    /// ## Append Subtree taken out of another Tree
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
    /// let mut a = DynTree::<_>::new(0);
    /// let [id1, id2] = a.root_mut().push_children([1, 2]);
    /// a.node_mut(&id1).push_children([3, 4]);
    ///
    /// let mut b = DaryTree::<4, _>::new(5);
    /// let id5 = b.root().idx();
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(&id6).push_child(8);
    /// b.node_mut(&id7).push_children([9, 10]);
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
    /// let n7 = b.node_mut(&id7).into_subtree();
    /// a.node_mut(&id2).append_child_tree(n7);
    ///
    /// let n1 = a.node_mut(&id1).into_subtree();
    /// b.node_mut(&id5).append_child_tree(n1);
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
    /// ## Append Another Tree
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
    /// let id1 = tree.node_mut(&id0).push_child(1);
    /// tree.node_mut(&id1).push_child(3);
    ///
    /// let mut b = BinaryTree::<_>::new(4);
    /// b.root_mut().push_child(7);
    ///
    /// let mut c = DaryTree::<4, _>::new(2);
    /// let [id5, id6] = c.root_mut().push_children([5, 6]);
    /// c.node_mut(&id5).push_child(8);
    /// c.node_mut(&id6).push_children([9, 10]);
    ///
    /// // merge b & c into tree
    ///
    /// let id4 = tree.node_mut(&id1).append_child_tree(b);
    /// let id2 = tree.node_mut(&id0).append_child_tree(c);
    ///
    /// assert_eq!(tree.node(&id2).data(), &2);
    /// assert_eq!(tree.node(&id4).data(), &4);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    /// ```
    pub fn append_child_tree(&mut self, subtree: impl SubTree<V::Item>) -> NodeIdx<V> {
        subtree.append_to_node_as_child(self, self.num_children())
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
    /// Further panics if the tree is of a variant with fixed children capacity,
    /// such as 2 for [`BinaryTree`] or `D` for [`DaryTree`] in general,
    /// and if this hard capacity is violated with the new sibling.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
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
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
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
    /// let mut n4 = tree.node_mut(&id4);
    /// n4.push_sibling(Side::Left, 7);
    /// n4.push_sibling(Side::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(&id6);
    /// n6.push_sibling(Side::Left, 9);
    /// n6.push_sibling(Side::Left, 10);
    /// let id12 = n6.push_sibling(Side::Right, 12);
    /// let id11 = n6.push_sibling(Side::Right, 11);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// assert_eq!(tree.node(&id12).data(), &12);
    /// assert_eq!(tree.node(&id11).data(), &11);
    /// ```
    pub fn push_sibling(&mut self, side: Side, value: V::Item) -> NodeIdx<V> {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let position = match side {
            Side::Left => self.sibling_idx(),
            Side::Right => self.sibling_idx() + 1,
        };

        let ptr = Self::insert_sibling_get_ptr(self.col, value, &parent_ptr, position);
        self.node_idx_for(&ptr)
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
    /// Further, the method might panic if the tree variant allows for a fixed number
    /// of children, as [`BinaryTree`] or any [`DaryTree`], and this capacity is exceeded.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
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
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
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
    /// let mut n4 = tree.node_mut(&id4);
    /// n4.push_sibling(Side::Left, 7);
    /// n4.push_sibling(Side::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(&id6);
    /// let [id9, id10] = n6.push_siblings(Side::Left, [9, 10]);
    /// let [id11, id12] = n6.push_siblings(Side::Right, [11, 12]);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// assert_eq!(tree.node(&id9).data(), &9);
    /// assert_eq!(tree.node(&id10).data(), &10);
    /// assert_eq!(tree.node(&id11).data(), &11);
    /// assert_eq!(tree.node(&id12).data(), &12);
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
            let sibling_ptr =
                Self::insert_sibling_get_ptr(self.col, sibling, &parent_ptr, position);
            position += 1;
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                &sibling_ptr,
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
    /// Further, the method might panic if the tree variant allows for a fixed number
    /// of children, as [`BinaryTree`] or any [`DaryTree`], and this capacity is exceeded.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
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
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
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
    /// let mut n4 = tree.node_mut(&id4);
    /// n4.push_sibling(Side::Left, 7);
    /// n4.push_sibling(Side::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(&id6);
    /// n6.extend_siblings(Side::Left, 9..=10).count();
    /// let idx: Vec<_> = n6.extend_siblings(Side::Right, 11..=12).collect();
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// assert_eq!(tree.node(&idx[0]).data(), &11);
    /// assert_eq!(tree.node(&idx[1]).data(), &12);
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
            let sibling_ptr =
                Self::insert_sibling_get_ptr(self.col, sibling, &parent_ptr, position);
            position += 1;
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                &sibling_ptr,
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
    /// Further panics if the tree is of a variant with fixed children capacity,
    /// such as 2 for [`BinaryTree`] or `D` for [`DaryTree`] in general,
    /// and if this hard capacity is violated with the new sibling.
    ///
    /// [`BinaryTree`]: crate::BinaryTree
    /// [`DaryTree`]: crate::DaryTree
    ///
    /// # Examples
    ///
    /// ## Append Subtree cloned-copied from another Tree
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
    /// let [_, id4] = a.node_mut(&id1).push_children([3, 4]);
    ///
    /// let mut b = DaryTree::<4, _>::new(5);
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(&id6).push_child(8);
    /// b.node_mut(&id7).push_children([9, 10]);
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
    /// let n6 = b.node(&id6).as_cloned_subtree();
    /// a.node_mut(&id4).append_sibling_tree(Side::Left, n6);
    ///
    /// let n7 = b.node(&id7).as_copied_subtree();
    /// a.node_mut(&id2).append_sibling_tree(Side::Right, n7);
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
    /// ## Append Subtree taken out of another Tree
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
    /// let mut a = DynTree::<_>::new(0);
    /// let [id1, id2] = a.root_mut().push_children([1, 2]);
    /// a.node_mut(&id1).push_children([3, 4]);
    ///
    /// let mut b = DaryTree::<4, _>::new(5);
    /// let [id6, id7] = b.root_mut().push_children([6, 7]);
    /// b.node_mut(&id6).push_child(8);
    /// b.node_mut(&id7).push_children([9, 10]);
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
    /// let n7 = b.node_mut(&id7).into_subtree();
    /// a.node_mut(&id2).append_sibling_tree(Side::Left, n7);
    ///
    /// let n1 = a.node_mut(&id1).into_subtree();
    /// b.node_mut(&id6).append_sibling_tree(Side::Right, n1);
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
    /// ## Append Another Tree
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
    /// let id1 = tree.node_mut(&id0).push_child(1);
    /// let id3 = tree.node_mut(&id1).push_child(3);
    ///
    /// let mut b = BinaryTree::<_>::new(4);
    /// b.root_mut().push_child(7);
    ///
    /// let mut c = DaryTree::<4, _>::new(2);
    /// let [id5, id6] = c.root_mut().push_children([5, 6]);
    /// c.node_mut(&id5).push_child(8);
    /// c.node_mut(&id6).push_children([9, 10]);
    ///
    /// // merge b & c into tree
    ///
    /// let id4 = tree.node_mut(&id3).append_sibling_tree(Side::Left, b);
    /// let id2 = tree.node_mut(&id1).append_sibling_tree(Side::Right, c);
    ///
    /// assert_eq!(tree.node(&id2).data(), &2);
    /// assert_eq!(tree.node(&id4).data(), &4);
    ///
    /// // validate the tree
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 4, 3, 5, 6, 7, 8, 9, 10]);
    /// ```
    pub fn append_sibling_tree(
        &mut self,
        side: Side,
        subtree: impl SubTree<V::Item>,
    ) -> NodeIdx<V> {
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
    ///
    /// // remove n4 downwards (removes 4 and 8)
    ///
    /// let data = tree.node_mut(&id4).remove();
    /// assert_eq!(data, 4);
    /// assert_eq!(tree.len(), 9);
    ///
    /// let root = tree.root();
    /// let values: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(values, [1, 2, 3, 5, 6, 7, 9, 10, 11]);
    ///
    /// // remove n3 downwards (3, 6, 7, 9, 10, 11)
    ///
    /// let data = tree.node_mut(&id3).remove();
    /// assert_eq!(data, 3);
    /// assert_eq!(tree.len(), 3);
    ///
    /// let root = tree.root();
    /// let values: Vec<_> = root.walk::<Bfs>().copied().collect();
    /// assert_eq!(values, [1, 2, 5]);
    ///
    /// // remove the root: clear the entire (remaining) tree
    ///
    /// let data = tree.get_root_mut().unwrap().remove();
    /// assert_eq!(data, 1);
    /// assert_eq!(tree.len(), 0);
    /// assert_eq!(tree.get_root(), None);
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
    /// let mut root = tree.root_mut();
    /// root.push_children([2, 3]);
    ///
    /// for c in 0..root.num_children() {
    ///     let mut node = root.child_mut(c).unwrap();
    ///
    ///     let val = *node.data();
    ///     let children = (0..val).map(|x| x + 1 + val);
    ///
    ///     let _ = node.extend_children(children).count();
    ///
    ///     for c in 0..node.num_children() {
    ///         let mut node = node.child_mut(c).unwrap();
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
    pub fn child_mut(&mut self, child_index: usize) -> Option<NodeMut<V, M, P>> {
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
    /// let mut n2 = tree.node_mut(&id2);
    /// n2.push_children([4, 5]);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(10);
    /// tree.node_mut(&id7).push_children([711, 712]);
    ///
    /// // push nodes 8 and 9 using children_mut of node 2
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// for mut child in n2.children_mut() {
    ///     let child_val = *child.data(); // 4 & 5
    ///     child.push_child(child_val + 4); // 8 & 9
    /// }
    ///
    /// // update values using children_mut of node 7
    ///
    /// let mut n7 = tree.node_mut(&id7);
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
        ChildrenMutIter::new(self.col, self.node_ptr.ptr())
    }

    /// Creates an iterator that yields mutable references to data of all nodes belonging to the subtree rooted at this node.
    ///
    /// The order of the elements is determined by the generic [`Traverser`] parameter `T`.
    /// Available implementations are:
    /// * [`Bfs`] for breadth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
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
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
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
    /// let mut n3 = tree.node_mut(&id3);
    /// {
    ///     let mut dfs = n3.walk_mut::<Dfs>();
    ///     assert_eq!(dfs.next(), Some(&mut 3));
    ///     assert_eq!(dfs.next(), Some(&mut 6)); // ...
    /// }
    ///
    /// let mut n2 = tree.node_mut(&id2);
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
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`walk_mut`], this method does require internal allocation.
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
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
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
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
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
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
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
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
    ///
    /// // remove any subtree rooted at a selected node
    /// // from the tree, and collect the node values
    /// // in the order of different traversals
    ///
    /// let n4 = tree.node_mut(&id4);
    /// let removed: Vec<_> = n4.into_walk::<PostOrder>().collect();
    /// assert_eq!(removed, [8, 4]);
    ///
    /// let remaining: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().copied().collect();
    /// assert_eq!(remaining, [1, 2, 3, 5, 6, 7, 9, 10, 11]);
    ///
    /// let n3 = tree.node_mut(&id3);
    /// let removed: Vec<_> = n3.into_walk::<Dfs>().collect();
    /// assert_eq!(removed, [3, 6, 9, 7, 10, 11]);
    ///
    /// let remaining: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().copied().collect();
    /// assert_eq!(remaining, [1, 2, 5]);
    ///
    /// let root = tree.get_root_mut().unwrap();
    /// let removed: Vec<_> = root.into_walk::<Bfs>().collect(); // empties the tree
    /// assert_eq!(removed, [1, 2, 5]);
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.get_root(), None);
    /// ```
    pub fn into_walk<T>(self) -> impl Iterator<Item = V::Item> + use<'a, T, V, M, P, MO>
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
    /// * [`Dfs`] for depth-first ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search))
    /// * [`PostOrder`] for post-order ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN))
    ///
    /// As opposed to [`into_walk`], this method does require internal allocation.
    /// Furthermore, it allows to attach node depths or sibling indices to the yield values.
    /// Please see the examples below.
    ///
    /// [`into_walk`]: crate::NodeMut::into_walk
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
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
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
    /// let [id2, id3] = root.push_children([2, 3]);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id4, _] = n2.push_children([4, 5]);
    ///
    /// tree.node_mut(&id4).push_child(8);
    ///
    /// let mut n3 = tree.node_mut(&id3);
    /// let [id6, id7] = n3.push_children([6, 7]);
    ///
    /// tree.node_mut(&id6).push_child(9);
    /// tree.node_mut(&id7).push_children([10, 11]);
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

    // subtree

    /// Creates a subtree view including this node as the root and all of its descendants with their orientation relative
    /// to this node.
    ///
    /// Consuming the created subtree in methods such as [`append_child_tree`] or [`append_sibling_tree`] will remove the
    /// subtree from this tree and move it to the target tree.
    /// Please see **Append Subtree taken out of another Tree** section of the examples of these methods.
    ///
    /// Otherwise, it has no impact on the tree.
    ///
    /// [`append_child_tree`]: crate::NodeMut::append_child_tree
    /// [`append_sibling_tree`]: crate::NodeMut::append_sibling_tree
    pub fn into_subtree(self) -> NodeMutAsSubTree<'a, V, M, P, MO> {
        NodeMutAsSubTree::new(self)
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
        let parent_ptr = self.node_ptr.clone();

        let child_ptr = self.col.push(value);

        let child = self.col.node_mut(&child_ptr);
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = self.col.node_mut(&parent_ptr);
        parent.next_mut().push(child_ptr.clone());

        child_ptr
    }

    fn insert_sibling_get_ptr(
        col: &mut Col<V, M, P>,
        value: V::Item,
        parent_ptr: &NodePtr<V>,
        position: usize,
    ) -> NodePtr<V> {
        let sibling_ptr = col.push(value);

        let child = col.node_mut(&sibling_ptr);
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = col.node_mut(parent_ptr);
        parent.next_mut().insert(position, sibling_ptr.clone());

        sibling_ptr
    }

    pub(crate) fn into_inner(self) -> (&'a mut Col<V, M, P>, NodePtr<V>) {
        (self.col, self.node_ptr)
    }

    fn parent_ptr(&self) -> Option<NodePtr<V>> {
        self.node().prev().get().cloned()
    }

    fn node_idx_for(&self, ptr: &NodePtr<V>) -> NodeIdx<V> {
        NodeIdx(orx_selfref_col::NodeIdx::new(self.col.memory_state(), ptr))
    }

    pub(crate) fn append_subtree_as_child(
        &mut self,
        subtree: impl IntoIterator<Item = (usize, V::Item)>,
        child_idx: usize,
    ) -> NodeIdx<V> {
        let mut iter = subtree.into_iter();
        let (mut current_depth, value) = iter.next().expect("tree is not empty");
        debug_assert_eq!(current_depth, 0);

        let idx = match child_idx == self.num_children() {
            true => self.push_child(value),
            false => {
                let ptr = Self::insert_sibling_get_ptr(self.col, value, &self.node_ptr, child_idx);
                self.node_idx_for(&ptr)
            }
        };

        let position = child_idx;
        let mut dst = self.child_mut(position).expect("child exists");

        for (depth, value) in iter {
            match depth > current_depth {
                true => debug_assert_eq!(depth, current_depth + 1, "dfs error in clone"),
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

        idx
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
    /// let mut a = tree.node_mut(&id_a);
    /// a.push_children(['c', 'd', 'e']);
    ///
    /// let mut b = tree.node_mut(&id_b);
    /// let [_, id_g] = b.push_children(['f', 'g']);
    ///
    /// let mut g = tree.node_mut(&id_g);
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
        self.node()
            .prev()
            .get()
            .cloned()
            .map(|p| NodeMut::new(self.col, p))
    }
}
