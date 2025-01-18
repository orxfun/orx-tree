use crate::{helpers::N, pinned_storage::PinnedStorage, MemoryPolicy, Tree, TreeVariant};

impl<V, M, P> IntoIterator for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    type Item = V::Item;

    type IntoIter = TreeIntoIter<V, <<P as PinnedStorage>::PinnedVec<V> as IntoIterator>::IntoIter>;

    /// Consumes the tree and creates an iterator over the data of the nodes of the tree in
    /// a deterministic but an arbitrary order.
    ///
    /// In order to take the values out of the tree in a particular order,
    /// you may use [`into_walk`] method on the root of the tree (or on any subtree) with the desired traverser.
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
    /// // |
    /// // 7
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
    /// n2.push_children([5, 6]);
    ///
    /// // transform the tree into an arbitrary order iterator
    ///
    /// let values: Vec<_> = tree.into_iter().collect();
    /// assert_eq!(values, [0, 1, 2, 3, 4, 7, 5, 6]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        let (col, _) = self.0.into_inner();
        let (pinned_vec, _, _) = col.into_inner();
        TreeIntoIter {
            iter: pinned_vec.into_iter(),
        }
    }
}

pub struct TreeIntoIter<V, I>
where
    V: TreeVariant,
    I: Iterator<Item = N<V>>,
{
    iter: I,
}

impl<V, I> Iterator for TreeIntoIter<V, I>
where
    V: TreeVariant,
    I: Iterator<Item = N<V>>,
{
    type Item = V::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().and_then(|mut node| node.take_data())
    }
}
