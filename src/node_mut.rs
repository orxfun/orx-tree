use crate::{
    helpers::{Col, N},
    iter::ChildrenMutIter,
    memory::{Auto, MemoryPolicy},
    node_ref::NodeRefCore,
    pinned_storage::{PinnedStorage, SplitRecursive},
    traversal::{
        enumerations::{DepthVal, Val},
        over::OverDepthPtr,
        over_mut::{OverItemInto, OverItemMut},
        post_order::iter_ptr::PostOrderIterPtr,
        traverser_core::TraverserCore,
        Over, OverData, OverMut,
    },
    tree_node_idx::INVALID_IDX_ERROR,
    tree_variant::RefsChildren,
    Dfs, NodeIdx, NodeRef, SubTree, Traverser, TreeVariant,
};
use core::marker::PhantomData;
use orx_selfref_col::{NodePtr, Refs};
use std::println;

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
pub enum SiblingSide {
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

    // /// Pushes nodes with given `children` as children of this node;
    // ///
    // /// # See also
    // ///
    // /// If the corresponding node indices of the children are required;
    // /// you may use [`grow`]:
    // ///
    // /// * `node.push_child(child);`
    // /// * `let child_idx = node.push_children([child]);`
    // ///
    // /// [`grow`]: crate::NodeMut::grow
    // ///
    // /// # Examples
    // ///
    // /// ```
    // /// use orx_tree::*;
    // ///
    // /// let mut tree = DynTree::<char>::new('a');
    // ///
    // /// let mut node = tree.get_root_mut().unwrap();
    // /// let b = node.push_child('b'); // b is the index of the node
    // /// node.push_children(['c', 'd', 'e']);
    // ///
    // /// assert_eq!(node.num_children(), 4);
    // /// ```
    // pub fn push_children<I>(&mut self, children: I)
    // where
    //     I: IntoIterator<Item = V::Item>,
    // {
    //     for x in children.into_iter() {
    //         self.push_child(x);
    //     }
    // }

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

    /// Pushes the subtree rooted at the given `subtree` node as a child of this node.
    ///
    /// The source `subtree` remains unchanged, the values are cloned into this tree.
    ///
    /// # See also
    ///
    /// See also [`push_tree_with`] which is identical to this method except that it re-uses
    /// a depth-first-search traverser, which otherwise is created temporarily within this
    /// method.
    ///
    /// [`push_tree_with`]: crate::NodeMut::push_tree_with
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // TREE a
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
    /// let [a1, a2] = a.root_mut().push_children([1, 2]);
    /// let [a3, a4] = a.node_mut(&a1).push_children([3, 4]);
    /// a.node_mut(&a3).push_child(7);
    /// let [a5, a6] = a.node_mut(&a2).push_children([5, 6]);
    /// a.node_mut(&a5).push_child(8);
    /// a.node_mut(&a6).push_children([9, 10]);
    ///
    /// let bfs: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// // TREE b
    /// //     10
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //  11     12
    /// //  ╱     ╱ | ╲
    /// // 13   14 15 16
    ///
    /// let mut b = DaryTree::<4, i32>::new(10);
    /// let [b11, b12] = b.root_mut().push_children([11, 12]);
    /// b.node_mut(&b11).push_child(13);
    /// b.node_mut(&b12).push_children([14, 15, 16]);
    ///
    /// let bfs: Vec<_> = b.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [10, 11, 12, 13, 14, 15, 16]);
    ///
    /// // Subtrees from tree b => Tree a
    /// // push subtree rooted at 12 as a child of node 4
    /// // push subtree rooted at 11 as a child of node 6
    /// //         0
    /// //        ╱ ╲
    /// //       ╱   ╲
    /// //      ╱     ╲
    /// //     ╱       ╲
    /// //    1         2
    /// //   ╱ ╲       ╱ ╲
    /// //  ╱   ╲     ╱   ╲
    /// // 3     4   5     6
    /// // |     |   |   ╱ | ╲
    /// // 7    12   8  9 10  11
    /// //    ╱ | ╲            |
    /// //   14 15 16         13
    ///
    /// a.node_mut(&a4).push_tree(&b.node(&b12));
    /// a.node_mut(&a6).push_tree(&b.node(&b11));
    ///
    /// let bfs: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(
    ///     bfs,
    ///     [0, 1, 2, 3, 4, 5, 6, 7, 12, 8, 9, 10, 11, 14, 15, 16, 13]
    /// );
    /// ```
    pub fn push_tree<V2, M2, P2>(&mut self, subtree: &impl NodeRef<'a, V2, M2, P2>)
    where
        V2: TreeVariant<Item = V::Item> + 'a,
        M2: MemoryPolicy,
        P2: PinnedStorage,
        V::Item: Clone,
    {
        let mut traverser = Dfs::<OverDepthPtr>::new();
        self.push_tree_with(subtree, &mut traverser);
    }

    /// Pushes the subtree rooted at the given `subtree` node as a child of this node.
    ///
    /// The source `subtree` remains unchanged, the values are cloned into this tree.
    ///
    /// # See also
    ///
    /// This method does not allocate and uses the provided depth-first-search `traverser`.
    /// See also [`push_tree`] which is identical to this method it creates the traverser
    /// temporarily within the method.
    ///
    /// [`push_tree`]: crate::NodeMut::push_tree
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // TREE a
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
    /// let [a1, a2] = a.root_mut().push_children([1, 2]);
    /// let [a3, a4] = a.node_mut(&a1).push_children([3, 4]);
    /// a.node_mut(&a3).push_child(7);
    /// let [a5, a6] = a.node_mut(&a2).push_children([5, 6]);
    /// a.node_mut(&a5).push_child(8);
    /// a.node_mut(&a6).push_children([9, 10]);
    ///
    /// let bfs: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// // TREE b
    /// //     10
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //  11     12
    /// //  ╱     ╱ | ╲
    /// // 13   14 15 16
    ///
    /// let mut b = DaryTree::<4, i32>::new(10);
    /// let [b11, b12] = b.root_mut().push_children([11, 12]);
    /// b.node_mut(&b11).push_child(13);
    /// b.node_mut(&b12).push_children([14, 15, 16]);
    ///
    /// let bfs: Vec<_> = b.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [10, 11, 12, 13, 14, 15, 16]);
    ///
    /// // Subtrees from tree b => Tree a
    /// // push subtree rooted at 12 as a child of node 4
    /// // push subtree rooted at 11 as a child of node 6
    /// //         0
    /// //        ╱ ╲
    /// //       ╱   ╲
    /// //      ╱     ╲
    /// //     ╱       ╲
    /// //    1         2
    /// //   ╱ ╲       ╱ ╲
    /// //  ╱   ╲     ╱   ╲
    /// // 3     4   5     6
    /// // |     |   |   ╱ | ╲
    /// // 7    12   8  9 10  11
    /// //    ╱ | ╲            |
    /// //   14 15 16         13
    ///
    /// let mut dfs = Traversal.dfs().with_depth(); // reusable traverser
    ///
    /// a.node_mut(&a4).push_tree_with(&b.node(&b12), &mut dfs);
    /// a.node_mut(&a6).push_tree_with(&b.node(&b11), &mut dfs);
    ///
    /// let bfs: Vec<_> = a.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(
    ///     bfs,
    ///     [0, 1, 2, 3, 4, 5, 6, 7, 12, 8, 9, 10, 11, 14, 15, 16, 13]
    /// );
    /// ```
    #[allow(clippy::missing_panics_doc)]
    pub fn push_tree_with<V2, M2, P2, O>(
        &mut self,
        subtree: &impl NodeRef<'a, V2, M2, P2>,
        traverser: &mut Dfs<O>,
    ) where
        V2: TreeVariant<Item = V::Item> + 'a,
        M2: MemoryPolicy,
        P2: PinnedStorage,
        O: Over<Enumeration = DepthVal>,
        V::Item: Clone,
    {
        #[inline(always)]
        fn data_of<V>(node_ptr: NodePtr<V>) -> V::Item
        where
            V: TreeVariant,
            V::Item: Clone,
        {
            (unsafe { &*node_ptr.ptr() })
                .data()
                .expect("node is active")
                .clone()
        }

        let storage = traverser.storage_mut();
        let mut iter =
            Dfs::<OverDepthPtr>::iter_ptr_with_storage(subtree.node_ptr().clone(), storage);
        let (mut current_depth, src_ptr) = iter.next().expect("tree is not empty");
        debug_assert_eq!(current_depth, 0);

        let position = self.num_children();
        self.push_child(data_of(src_ptr));
        let mut dst = self.child_mut(position).expect("child exists");

        for (depth, ptr) in iter {
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
            dst.push_child(data_of(ptr));
            dst = dst.into_child_mut(position).expect("child exists");
            current_depth = depth;
        }
    }

    pub fn connect_as_child(&mut self, subtree: impl SubTree<V::Item>) {
        todo!()
    }

    // growth - horizontally

    /// Pushes a sibling with the given `value`:
    ///
    /// * as the immediate left-sibling of this node when `side` is [`SiblingSide::Left`],
    /// * as the immediate right-sibling of this node when `side` is [`SiblingSide::Right`],
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
    /// n4.push_sibling(SiblingSide::Left, 7);
    /// n4.push_sibling(SiblingSide::Right, 8);
    ///
    /// let mut n6 = tree.node_mut(&id6);
    /// n6.push_sibling(SiblingSide::Left, 9);
    /// n6.push_sibling(SiblingSide::Left, 10);
    /// n6.push_sibling(SiblingSide::Right, 12);
    /// n6.push_sibling(SiblingSide::Right, 11);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    /// ```
    pub fn push_sibling(&mut self, side: SiblingSide, value: V::Item) {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let position = match side {
            SiblingSide::Left => self.sibling_idx(),
            SiblingSide::Right => self.sibling_idx() + 1,
        };

        self.insert_sibling_get_ptr(value, &parent_ptr, position);
    }

    /// Pushes the nodes with the given data `siblings`:
    ///
    /// * as the immediate left,-siblings of this node when `side` is [`SiblingSide::Left`],
    /// * as the immediate right-siblings of this node when `side` is [`SiblingSide::Right`].
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
    /// # See also
    ///
    /// If the corresponding node indices of the siblings are required;
    /// you may use [`grow_siblings`] or [`grow_siblings_iter`].
    ///
    /// [`grow_siblings`]: crate::NodeMut::grow_siblings
    /// [`grow_siblings_iter`]: crate::NodeMut::grow_siblings_iter
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
    /// // let mut n4 = tree.node_mut(&id4);
    /// // n4.push_sibling(7, SiblingSide::Left);
    /// // n4.push_sibling(8, SiblingSide::Right);
    ///
    //     /// // let mut n6 = tree.node_mut(&id6);
    /// // n6.push_siblings([9, 10], SiblingSide::Left);
    /// // n6.push_siblings([11, 12], SiblingSide::Right);
    ///
    //     /// // let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// // assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    /// ```
    pub fn push_siblings<I>(&mut self, siblings: I, side: SiblingSide)
    where
        I: IntoIterator<Item = V::Item>,
    {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let mut position = match side {
            SiblingSide::Left => self.sibling_idx(),
            SiblingSide::Right => self.sibling_idx() + 1,
        };

        for sibling in siblings.into_iter() {
            self.insert_sibling_get_ptr(sibling, &parent_ptr, position);
            position += 1;
        }
    }

    /// Pushes the nodes with the given data `siblings`:
    ///
    /// * as the immediate left,-siblings of this node when `side` is [`SiblingSide::Left`],
    /// * as the immediate right-siblings of this node when `side` is [`SiblingSide::Right`].
    ///
    /// Returns an array of indices of the nodes added, in the same order of the `siblings` data.
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
    /// # See also
    ///
    /// See [`grow_siblings_iter`] to push **non-const** number of children
    /// and obtain corresponding node indices.
    ///
    /// If the corresponding node indices of the siblings are not required;
    /// you may use [`push_sibling`] or [`push_siblings`].
    ///
    /// [`push_sibling`]: crate::NodeMut::push_sibling
    /// [`push_siblings`]: crate::NodeMut::push_siblings
    /// [`grow_siblings_iter`]: crate::NodeMut::grow_siblings_iter
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
    /// let [id7] = n4.grow_siblings([7], SiblingSide::Left);
    /// let [id8] = n4.grow_siblings([8], SiblingSide::Right);
    ///
    /// let mut n6 = tree.node_mut(&id6);
    /// let [id9, id10] = n6.grow_siblings([9, 10], SiblingSide::Left);
    /// let [id11, id12] = n6.grow_siblings([11, 12], SiblingSide::Right);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// // as grow methods, grow_siblings method allows us to cache indices
    /// // of new nodes immediately
    ///
    /// assert_eq!(tree.node(&id7).data(), &7);
    /// assert_eq!(tree.node(&id8).data(), &8);
    /// assert_eq!(tree.node(&id9).data(), &9);
    /// assert_eq!(tree.node(&id10).data(), &10);
    /// assert_eq!(tree.node(&id11).data(), &11);
    /// assert_eq!(tree.node(&id12).data(), &12);
    /// ```
    pub fn grow_siblings<const N: usize>(
        &mut self,
        siblings: [V::Item; N],
        side: SiblingSide,
    ) -> [NodeIdx<V>; N] {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let mut position = match side {
            SiblingSide::Left => self.sibling_idx(),
            SiblingSide::Right => self.sibling_idx() + 1,
        };

        siblings.map(|sibling| {
            let sibling_ptr = self.insert_sibling_get_ptr(sibling, &parent_ptr, position);
            position += 1;
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                &sibling_ptr,
            ))
        })
    }

    /// Pushes the nodes with the given data `siblings`:
    ///
    /// * as the immediate left,-siblings of this node when `side` is [`SiblingSide::Left`],
    /// * as the immediate right-siblings of this node when `side` is [`SiblingSide::Right`].
    ///
    /// Returns an iterator of indices of the nodes added, in the same order of the `siblings` data.
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
    /// # See also
    ///
    /// See [`grow_siblings`] to push a **const** number of children
    /// and obtain corresponding node indices.
    ///
    /// If the corresponding node indices of the siblings are not required;
    /// you may use [`push_sibling`] or [`push_siblings`].
    ///
    /// [`push_sibling`]: crate::NodeMut::push_sibling
    /// [`push_siblings`]: crate::NodeMut::push_siblings
    /// [`grow_siblings`]: crate::NodeMut::grow_siblings
    ///
    /// # Examples
    ///
    /// Following example demonstrates one way to build an arbitrary depth tree with a special data structure systematically
    /// using the `extend_children` method.
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
    /// let [id7] = n4.grow_siblings([7], SiblingSide::Left);
    /// let [id8] = n4.grow_siblings([8], SiblingSide::Right);
    ///
    /// let mut n6 = tree.node_mut(&id6);
    /// let indices: Vec<_> = n6.grow_siblings_iter(9..11, SiblingSide::Left).collect();
    /// let id9 = &indices[0];
    /// let id10 = &indices[1];
    ///
    /// let indices: Vec<_> = n6.grow_siblings_iter(11..13, SiblingSide::Right).collect();
    /// let id11 = &indices[0];
    /// let id12 = &indices[1];
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [1, 2, 3, 7, 4, 8, 5, 9, 10, 6, 11, 12]);
    ///
    /// // as grow methods, grow_siblings method allows us to cache indices
    /// // of new nodes immediately
    ///
    /// assert_eq!(tree.node(&id7).data(), &7);
    /// assert_eq!(tree.node(&id8).data(), &8);
    /// assert_eq!(tree.node(&id9).data(), &9);
    /// assert_eq!(tree.node(&id10).data(), &10);
    /// assert_eq!(tree.node(&id11).data(), &11);
    /// assert_eq!(tree.node(&id12).data(), &12);
    /// ```
    pub fn grow_siblings_iter<'b, I>(
        &'b mut self,
        siblings: I,
        side: SiblingSide,
    ) -> impl Iterator<Item = NodeIdx<V>> + 'b + use<'b, 'a, I, V, M, P, MO>
    where
        I: IntoIterator<Item = V::Item>,
        I::IntoIter: 'b,
    {
        let parent_ptr = self
            .parent_ptr()
            .expect("Cannot push sibling to the root node");

        let mut position = match side {
            SiblingSide::Left => self.sibling_idx(),
            SiblingSide::Right => self.sibling_idx() + 1,
        };

        siblings.into_iter().map(move |sibling| {
            let sibling_ptr = self.insert_sibling_get_ptr(sibling, &parent_ptr, position);
            position += 1;
            NodeIdx(orx_selfref_col::NodeIdx::new(
                self.col.memory_state(),
                &sibling_ptr,
            ))
        })
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
    /// See [`grow`] and [`extend_children`] methods to see an alternative tree building approach which makes
    /// use of node indices.
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`extend_children`]: crate::NodeMut::extend_children
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
        &mut self,
        value: V::Item,
        parent_ptr: &NodePtr<V>,
        position: usize,
    ) -> NodePtr<V> {
        let sibling_ptr = self.col.push(value);

        let child = self.col.node_mut(&sibling_ptr);
        child.prev_mut().set_some(parent_ptr.clone());

        let parent = self.col.node_mut(parent_ptr);
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
    /// See [`grow`] and [`extend_children`] methods to see an alternative tree building approach which makes
    /// use of node indices.
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`extend_children`]: crate::NodeMut::extend_children
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
    /// See [`grow`] and [`extend_children`] methods to see an alternative tree building approach which makes
    /// use of node indices.
    ///
    /// [`grow`]: crate::NodeMut::grow
    /// [`extend_children`]: crate::NodeMut::extend_children
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

#[test]
fn abc() {
    use crate::*;
    use alloc::vec;
    use alloc::vec::Vec;

    //      0
    //     ╱ ╲
    //    ╱   ╲
    //   1     2
    //  ╱ ╲   ╱ ╲
    // 3   4 5   6
    // |     |  ╱ ╲
    // 7     8 9   10

    let mut idx = vec![];

    let mut tree = DynTree::<_>::new(0);

    let mut root = tree.root_mut();
    idx.push(root.idx());
    idx.extend(root.extend_children(1..=2));

    let mut n1 = tree.node_mut(&idx[1]);
    idx.extend(n1.extend_children([3, 4]));

    let mut n2 = tree.node_mut(&idx[2]);
    idx.extend(n2.extend_children(5..=6));

    idx.push(tree.node_mut(&idx[3]).push_child(7));

    idx.push(tree.node_mut(&idx[5]).push_child(8));
    idx.extend(tree.node_mut(&idx[6]).extend_children([9, 10]));

    // validate the tree

    let root = tree.root();

    let bfs: Vec<_> = root.walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let dfs: Vec<_> = root.walk::<Dfs>().copied().collect();
    assert_eq!(dfs, [0, 1, 3, 7, 4, 2, 5, 8, 6, 9, 10]);
}
