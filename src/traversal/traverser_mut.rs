use super::{
    over_mut::{OverItemInto, OverItemMut, OverMut},
    Traverser,
};
use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage, NodeMut, TreeVariant};

/// A mutable tree traverser which walks over a given node and all of its descendants by its `iter_mut`
/// and `into_iter` methods.
///
///
///
/// The only argument of the [`iter_mut`] method is the `node` which is considered to be the root of the
/// tree composed of the the given node and all of its descendants.
///
/// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
/// breadth-first are the most well-known traversals.
///
/// All traverser types implement Default, and hence, can be created using the default function.
/// However, a more convenient to create them is to use the [`Traversal`] factory type.
///
/// Typically, a traverser holds its temporary or internal working data, and therefore, it might be
/// used once or many times to traverse trees without requiring additional allocation.
/// In other words, a traverser allocates the memory it requires only once when it is created;
/// and re-uses the same memory over and over for all the `iter_mut` calls.
///
/// [`iter_mut`]: crate::traversal::TraverserMut::iter_mut
/// [`Traversal`]: crate::Traversal
///
/// # Yields
///
/// The return value of the iterations depend on the second generic parameter of the traverser which implements
/// the [`OverMut`] trait. The following is the complete list of implementations and the corresponding item type
/// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
/// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
/// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
/// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
///
/// | Over | Yields | Depth First Example |
/// |---|---|---|
/// | [`OverData`] | &mut data | `Traversal.dfs()` |
/// | [`OverDepthData`] | (depth, &mut data) | `Traversal.dfs().with_depth()` |
/// | [`OverSiblingIdxData`] | (sibling_idx, &mut data) | `Traversal.dfs().with_sibling_idx()` |
/// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, &mut data) | `Traversal.with_depth().with_sibling_idx()` |
///
/// [`Traversal`]: crate::traversal::Traversal
/// [`OverMut`]: crate::traversal::OverMut
/// [`OverData`]: crate::traversal::OverData
/// [`OverDepthData`]: crate::traversal::OverDepthData
/// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
/// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
///
/// # Examples
///
/// ```
/// use orx_tree::*;
///
/// //     1
/// //    ╱
/// //   2
/// //  ╱ ╲
/// // 3   4
/// // |
/// // 5
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
/// let [id2] = root.grow([2]);
///
/// let mut n2 = id2.node_mut(&mut tree);
/// let [id3, _] = n2.grow([3, 4]);
///
/// id3.node_mut(&mut tree).push(5);
///
/// // create & allocate traverser once
///
/// let mut dfs = Traversal.dfs(); // OR: Dfs::<_, OverData>::default()
///
/// // re-use it multiple times for iter or iter_mut or into_iter
///
/// let mut root = tree.root_mut().unwrap();
/// for (i, data) in dfs.iter_mut(&mut root).enumerate() {
///     *data += 100 * i as i32;
/// }
///
/// let root = tree.root().unwrap();
/// let values: Vec<_> = dfs.iter(&root).copied().collect();
/// assert_eq!(values, [1, 102, 203, 305, 404]);
///
/// let n3 = id3.node(&tree);
/// let values: Vec<_> = dfs.iter(&n3).copied().collect();
/// assert_eq!(values, [203, 305]);
///
/// // create a traverser to yield (depth, sibling_idx, data) rather than data
///
/// let mut dfs = Traversal.dfs().with_depth().with_sibling_idx();
///
/// let mut n3 = id3.node_mut(&mut tree);
/// for (depth, sibling_idx, data) in dfs.iter_mut(&mut n3) {
///     *data += 10000 * (depth + sibling_idx) as i32;
/// }
///
/// let root = tree.root().unwrap();
/// let values: Vec<_> = dfs.iter(&root).map(|(_, _, data)| *data).collect();
/// assert_eq!(values, [1, 102, 203, 10305, 404]);
/// ```
pub trait TraverserMut<V, O>: Traverser<V, O>
where
    V: TreeVariant,
    O: OverMut<V>,
{
    /// Returns a mutable iterator which yields all nodes including the `node` and all its descendants; i.e.,
    /// all nodes of the subtree rooted at the given `node`.
    ///
    /// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
    /// breadth-first are the most well-known traversals.
    ///
    /// Typically, the `iter` or `iter_mut` or `into_iter` call of a traverser does not require any memory allocation.
    ///
    /// # Yields
    ///
    /// The return value of the iterations depend on the second generic parameter of the traverser which implements
    /// the [`OverMut`] trait. The following is the complete list of implementations and the corresponding item type
    /// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
    /// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
    /// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
    /// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
    ///
    /// | Over | Yields | Depth First Example |
    /// |---|---|---|
    /// | [`OverData`] | &mut data | `Traversal.dfs()` |
    /// | [`OverDepthData`] | (depth, &mut data) | `Traversal.dfs().with_depth()` |
    /// | [`OverSiblingIdxData`] | (sibling_idx, &mut data) | `Traversal.dfs().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, &mut data) | `Traversal.with_depth().with_sibling_idx()` |
    ///
    /// [`Traversal`]: crate::traversal::Traversal
    /// [`OverMut`]: crate::traversal::OverMut
    /// [`OverData`]: crate::traversal::OverData
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    /// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
    /// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //     1
    /// //    ╱
    /// //   2
    /// //  ╱ ╲
    /// // 3   4
    /// // |
    /// // 5
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// let [id2] = root.grow([2]);
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// let [id3, _] = n2.grow([3, 4]);
    ///
    /// id3.node_mut(&mut tree).push(5);
    ///
    /// // create & allocate traverser once
    ///
    /// let mut dfs = Traversal.dfs(); // OR: Dfs::<_, OverData>::default()
    ///
    /// // re-use it multiple times for iter or iter_mut or into_iter
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// for (i, data) in dfs.iter_mut(&mut root).enumerate() {
    ///     *data += 100 * i as i32;
    /// }
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = dfs.iter(&root).copied().collect();
    /// assert_eq!(values, [1, 102, 203, 305, 404]);
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = dfs.iter(&n3).copied().collect();
    /// assert_eq!(values, [203, 305]);
    ///
    /// // create a traverser to yield (depth, sibling_idx, data) rather than data
    ///
    /// let mut dfs = Traversal.dfs().with_depth().with_sibling_idx();
    ///
    /// let mut n3 = id3.node_mut(&mut tree);
    /// for (depth, sibling_idx, data) in dfs.iter_mut(&mut n3) {
    ///     *data += 10000 * (depth + sibling_idx) as i32;
    /// }
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = dfs.iter(&root).map(|(_, _, data)| *data).collect();
    /// assert_eq!(values, [1, 102, 203, 10305, 404]);
    /// ```
    fn iter_mut<'a, M, P>(
        &mut self,
        node: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: 'a,
        Self: 'a;

    /// Returns an iterator which:
    ///
    /// * traverses all nodes including the `node` and its descendants; i.e.,
    ///   all nodes of the subtree rooted at the given `node`,
    /// * removes the traversed nodes from the tree, and
    /// * yields their values.
    ///
    /// Once the iterator is consumed, the tree will be missing the subtree rooted at the given `node`.
    /// If the given node is the root of the tree, the tree will be empty after this call.
    ///
    /// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
    /// breadth-first are the most well-known traversals.
    ///
    /// Typically, the `iter` or `iter_mut` or `into_iter` call of a traverser does not require any memory allocation.
    ///
    /// # Yields
    ///
    /// The return value of the iterations depend on the second generic parameter of the traverser which implements
    /// the [`OverMut`] trait. The following is the complete list of implementations and the corresponding item type
    /// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
    /// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
    /// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
    /// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
    ///
    /// Importantly note that, since the created iterators remove the nodes from the tree, the "data" below represents
    /// the owned data taken out of the corresponding node, and hence, out of the tree.
    ///
    /// | Over | Yields | Depth First Example |
    /// |---|---|---|
    /// | [`OverData`] | data | `Traversal.dfs()` |
    /// | [`OverDepthData`] | (depth, data) | `Traversal.dfs().with_depth()` |
    /// | [`OverSiblingIdxData`] | (sibling_idx, data) | `Traversal.dfs().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, data) | `Traversal.with_depth().with_sibling_idx()` |
    ///
    /// [`Traversal`]: crate::traversal::Traversal
    /// [`OverMut`]: crate::traversal::OverMut
    /// [`OverData`]: crate::traversal::OverData
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    /// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
    /// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
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
    /// let id1 = root.idx();
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
    /// // create & allocate traverser once
    ///
    /// let mut post_order = Traversal.post_order(); // OR: PostOrder::<_, OverData>::default()
    ///
    /// // re-use it multiple times for iter or iter_mut or into_iter
    /// // here with into_iter, we remove node 3 and its descendants
    /// // collect the removed values into a vector in the traversal's order
    ///
    /// let n3 = id3.node_mut(&mut tree);
    /// let removed: Vec<_> = post_order.into_iter(n3).collect();
    /// assert_eq!(removed, [9, 6, 10, 11, 7, 3]);
    ///
    /// let root = id1.node(&tree);
    /// let remaining_values: Vec<_> = post_order.iter(&root).copied().collect();
    /// assert_eq!(remaining_values, [8, 4, 5, 2, 1]);
    ///
    /// // let's remove root and its descendants (empty the tree)
    /// // and collect remaining nodes in the traversal's order
    ///
    /// let root = id1.node_mut(&mut tree);
    /// let removed: Vec<_> = post_order.into_iter(root).collect();
    /// assert_eq!(removed, [8, 4, 5, 2, 1]);
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.root(), None);
    /// ```
    fn into_iter<'a, M, P>(
        &mut self,
        node: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: 'a,
        Self: 'a,
    {
        core::iter::empty()
    }
}
