use crate::{
    memory::MemoryPolicy, pinned_storage::PinnedStorage, traversal::over::OverDepthPtr, Dfs,
    NodeRef, Traverser, Tree, TreeVariant,
};

impl<V, M, P> Clone for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
    V::Item: Clone,
{
    /// Clones the tree.
    ///
    /// # See also
    ///
    /// Note that cloning the tree is implemented using a depth-first traversal which uses an internal
    /// stack that is allocated and dropped at the end of the clone operation.
    /// In use cases where we repeatedly traverse over nodes, we can avoid allocation by creating the
    /// traverser only once and reusing it with methods with a "_with" suffix, such as [`clone_with`].
    ///
    /// [`clone_with`]: crate::NodeRef::clone_with
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
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut();
    /// let [id1, id2] = root.grow([1, 2]);
    ///
    /// let mut n1 = tree.node_mut(&id1);
    /// let [id3, _] = n1.grow([3, 4]);
    ///
    /// tree.node_mut(&id3).push(7);
    ///
    /// let mut n2 = tree.node_mut(&id2);
    /// let [id5, id6] = n2.grow([5, 6]);
    ///
    /// tree.node_mut(&id5).push(8);
    /// tree.node_mut(&id6).extend([9, 10]);
    ///
    /// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// // clone the entire tree
    ///
    /// let clone = tree.clone();
    /// let bfs: Vec<_> = clone.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
    ///
    /// // indices are valid only for their trees
    ///
    /// let indices: Vec<_> = clone.root().indices::<Bfs>().collect();
    ///
    /// assert_eq!(tree.get_node(&indices[2]), None);
    /// assert_eq!(tree.try_node(&indices[2]), Err(NodeIdxError::OutOfBounds));
    ///
    /// assert_eq!(clone.get_node(&id2), None);
    /// assert_eq!(clone.try_node(&id2), Err(NodeIdxError::OutOfBounds));
    ///
    /// assert_eq!(clone.node(&indices[2]).data(), &2);
    /// ```
    fn clone(&self) -> Self {
        match self.get_root() {
            None => Self::empty(),
            Some(root) => {
                let mut traverser = Dfs::<OverDepthPtr>::new();
                let mut tree = Self::new(root.data().clone());

                for child in root.children() {
                    tree.root_mut().push_tree_with(&child, &mut traverser);
                }

                tree
            }
        }
    }
}
