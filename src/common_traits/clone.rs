use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage, NodeRef, Tree, TreeVariant};

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
    /// * [`clone_as_tree`]: to clone a subtree rooted at a given node as a separate tree.
    ///
    /// [`clone_as_tree`]: crate::NodeRef::clone_as_tree
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
                let mut tree = Self::new(root.data().clone());

                for child in root.children() {
                    tree.root_mut().append_child_tree(child.as_cloned_subtree());
                }

                tree
            }
        }
    }
}
