use crate::{helpers::N, pinned_storage::PinnedStorage, MemoryPolicy, Tree, TreeVariant};
use core::iter::FusedIterator;
use orx_pinned_vec::PinnedVec;

// owned

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
    /// [`into_walk`]: crate::NodeMut::into_walk
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

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(mut node) => {
                    if node.is_active() {
                        return node.take_data();
                    }
                }
            }
        }
    }
}

impl<V, I> FusedIterator for TreeIntoIter<V, I>
where
    V: TreeVariant,
    I: Iterator<Item = N<V>>,
{
}

// ref

impl<'a, V, M, P> IntoIterator for &'a Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    type Item = &'a V::Item;

    type IntoIter =
        TreeIter<'a, V, <<P as PinnedStorage>::PinnedVec<V> as PinnedVec<N<V>>>::Iter<'a>>;

    /// Creates an iterator over references to the data of the nodes of the tree in
    /// a deterministic but an arbitrary order.
    ///
    /// In order to iterate over the values the tree nodes in a particular order,
    /// you may use [`walk`] method on the root of the tree (or on any subtree) with the desired traverser.
    ///
    /// [`walk`]: crate::NodeRef::walk
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
    /// // iterate over the tree in an arbitrary order
    ///
    /// let values: Vec<_> = (&tree).into_iter().copied().collect();
    /// assert_eq!(values, [0, 1, 2, 3, 4, 7, 5, 6]);
    ///
    /// // since Tree auto-implements orx_iterable::Collection
    /// let values: Vec<_> = tree.iter().copied().collect();
    /// assert_eq!(values, [0, 1, 2, 3, 4, 7, 5, 6]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        TreeIter {
            iter: self.0.nodes().iter(),
        }
    }
}

pub struct TreeIter<'a, V, I>
where
    V: TreeVariant + 'a,
    I: Iterator<Item = &'a N<V>>,
{
    iter: I,
}

impl<'a, V, I> Iterator for TreeIter<'a, V, I>
where
    V: TreeVariant + 'a,
    I: Iterator<Item = &'a N<V>>,
{
    type Item = &'a V::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(node) => {
                    if node.is_active() {
                        return node.data();
                    }
                }
            }
        }
    }
}

impl<'a, V, I> FusedIterator for TreeIter<'a, V, I>
where
    V: TreeVariant + 'a,
    I: Iterator<Item = &'a N<V>>,
{
}

// mut

impl<'a, V, M, P> IntoIterator for &'a mut Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    type Item = &'a mut V::Item;

    type IntoIter =
        TreeIterMut<'a, V, <<P as PinnedStorage>::PinnedVec<V> as PinnedVec<N<V>>>::IterMut<'a>>;

    /// Creates a mutable iterator over references to the data of the nodes of the tree in
    /// a deterministic but an arbitrary order.
    ///
    /// In order to iterate over the values the tree nodes in a particular order,
    /// you may use [`walk_mut`] method on the root of the tree (or on any subtree) with the desired traverser.
    ///
    /// [`walk_mut`]: crate::NodeMut::walk_mut
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
    /// // mutably iterate over the tree in an arbitrary order
    ///
    /// for x in (&mut tree).into_iter() {
    ///     *x += 5;
    /// }
    ///
    /// // since Tree auto-implements orx_iterable::CollectionMut
    ///
    /// for x in tree.iter_mut() {
    ///     *x += 5;
    /// }
    ///
    /// // validate
    ///
    /// let values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(values, [10, 11, 12, 13, 14, 15, 16, 17]);
    /// ```
    fn into_iter(self) -> Self::IntoIter {
        TreeIterMut {
            iter: self.0.nodes_mut().iter_mut(),
        }
    }
}

pub struct TreeIterMut<'a, V, I>
where
    V: TreeVariant + 'a,
    I: Iterator<Item = &'a mut N<V>>,
{
    iter: I,
}

impl<'a, V, I> Iterator for TreeIterMut<'a, V, I>
where
    V: TreeVariant + 'a,
    I: Iterator<Item = &'a mut N<V>>,
{
    type Item = &'a mut V::Item;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => return None,
                Some(node) => {
                    if node.is_active() {
                        return node.data_mut();
                    }
                }
            }
        }
    }
}

impl<'a, V, I> FusedIterator for TreeIterMut<'a, V, I>
where
    V: TreeVariant + 'a,
    I: Iterator<Item = &'a mut N<V>>,
{
}
