use super::over::{Over, OverItem};
use crate::{helpers::N, NodeRef, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// A tree traverser which walks over a given node and all of its descendants by its `iter` method.
///
/// The only argument of the [`iter`] method is the `node` which is considered to be the root of the
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
/// and re-uses the same memory over and over for all the `iter` calls.
///
/// [`iter`]: crate::traversal::Traverser::iter
/// [`Traversal`]: crate::Traversal
///
/// # Yields
///
/// The return value of the iterations depend on the second generic parameter of the traverser which implements
/// the [`Over`] trait. The following is the complete list of implementations and the corresponding item type
/// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
/// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
/// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
/// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
///
/// | Over | Yields | Depth First Example |
/// |---|---|---|
/// | [`OverData`] | &data | `Traversal.dfs()` |
/// | [`OverDepthData`] | (depth, &data) | `Traversal.dfs().with_depth()` |
/// | [`OverSiblingIdxData`] | (sibling_idx, &data) | `Traversal.dfs().with_sibling_idx()` |
/// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, &data) | `Traversal.with_depth().with_sibling_idx()` |
/// | [`OverNode`] | Node | `Traversal.dfs().over_nodes()` |
/// | [`OverDepthNode`] | (depth, Node) | `Traversal.dfs().over_nodes().with_depth()` |
/// | [`OverSiblingIdxNode`] | (sibling_idx, Node) | `Traversal.dfs().over_nodes().with_sibling_idx()` |
/// | [`OverDepthSiblingIdxNode`] | (depth, sibling_idx, Node) | `Traversal.dfs().over_nodes().with_depth().with_sibling_idx()` |
///
/// [`Traversal`]: crate::traversal::Traversal
/// [`Over`]: crate::traversal::Over
/// [`OverData`]: crate::traversal::OverData
/// [`OverDepthData`]: crate::traversal::OverDepthData
/// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
/// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
/// [`OverNode`]: crate::traversal::OverNode
/// [`OverDepthNode`]: crate::traversal::OverDepthNode
/// [`OverSiblingIdxNode`]: crate::traversal::OverSiblingIdxNode
/// [`OverDepthSiblingIdxNode`]: crate::traversal::OverDepthSiblingIdxNode
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
/// // re-use it multiple times for iter (or iter_mut methods when possible)
///
/// let root = tree.root().unwrap();
/// let values: Vec<_> = dfs.iter(&root).copied().collect();
/// assert_eq!(values, [1, 2, 3, 5, 4]);
///
/// let n3 = id3.node(&tree);
/// let values: Vec<_> = dfs.iter(&n3).copied().collect();
/// assert_eq!(values, [3, 5]);
///
/// // create a traverser to yield (depth, node) rather than data
///
/// let mut dfs = Traversal.dfs().over_nodes().with_depth();
///
/// let mut iter = dfs.iter(&n3);
///
/// let (depth3, n3) = iter.next().unwrap();
/// assert_eq!(n3.data(), &3);
/// assert_eq!(n3.num_children(), 1);
/// assert_eq!(n3.parent().map(|x| *x.data()), Some(2));
/// assert_eq!(depth3, 0); // as it is the root of the traversed subtree
///
/// let (depth5, n5) = iter.next().unwrap();
/// assert_eq!(n5.data(), &5);
/// assert_eq!(n5.num_children(), 0);
/// assert_eq!(n5.parent().map(|x| *x.data()), Some(3));
/// assert_eq!(depth5, 1);
/// ```
pub trait Traverser<V, O>: Default
where
    V: TreeVariant,
    O: Over<V>,
{
    /// Transformed version of the traverser from creating iterators over `O` to `O2`.
    type IntoOver<O2>
    where
        O2: Over<V>;

    /// Returns an iterator which yields all nodes including the `node` and all its descendants; i.e.,
    /// all nodes of the subtree rooted at the given `node`.
    ///
    /// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
    /// breadth-first are the most well-known traversals.
    ///
    /// Typically, the `iter` call of a traverser does not require any memory allocation.
    ///
    /// # Yields
    ///
    /// The return value of the iterations depend on the second generic parameter of the traverser which implements
    /// the [`Over`] trait. The following is the complete list of implementations and the corresponding item type
    /// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
    /// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
    /// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
    /// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
    ///
    /// | Over | Yields | Depth First Example |
    /// |---|---|---|
    /// | [`OverData`] | &data | `Traversal.dfs()` |
    /// | [`OverDepthData`] | (depth, &data) | `Traversal.dfs().with_depth()` |
    /// | [`OverSiblingIdxData`] | (sibling_idx, &data) | `Traversal.dfs().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, &data) | `Traversal.with_depth().with_sibling_idx()` |
    /// | [`OverNode`] | Node | `Traversal.dfs().over_nodes()` |
    /// | [`OverDepthNode`] | (depth, Node) | `Traversal.dfs().over_nodes().with_depth()` |
    /// | [`OverSiblingIdxNode`] | (sibling_idx, Node) | `Traversal.dfs().over_nodes().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxNode`] | (depth, sibling_idx, Node) | `Traversal.dfs().over_nodes().with_depth().with_sibling_idx()` |
    ///
    /// [`Traversal`]: crate::traversal::Traversal
    /// [`OverData`]: crate::traversal::OverData
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    /// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
    /// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
    /// [`OverNode`]: crate::traversal::OverNode
    /// [`OverDepthNode`]: crate::traversal::OverDepthNode
    /// [`OverSiblingIdxNode`]: crate::traversal::OverSiblingIdxNode
    /// [`OverDepthSiblingIdxNode`]: crate::traversal::OverDepthSiblingIdxNode
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
    /// // |╲
    /// // 5 6
    ///
    /// let mut tree = DynTree::<i32>::new(1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// let [id2] = root.grow([2]);
    ///
    /// let mut n2 = id2.node_mut(&mut tree);
    /// let [id3, _] = n2.grow([3, 4]);
    ///
    /// id3.node_mut(&mut tree).extend([5, 6]);
    ///
    /// // create & allocate traverser once
    ///
    /// let mut post_order = Traversal.post_order(); // OR: PostOrder::<_, OverData>::default()
    ///
    /// // re-use it multiple times for iter (or iter_mut methods when possible)
    ///
    /// let root = tree.root().unwrap();
    /// let values: Vec<_> = post_order.iter(&root).copied().collect();
    /// assert_eq!(values, [5, 6, 3, 4, 2, 1]);
    ///
    /// let n3 = id3.node(&tree);
    /// let values: Vec<_> = post_order.iter(&n3).copied().collect();
    /// assert_eq!(values, [5, 6, 3]);
    ///
    /// // create a traverser to yield (depth, node) rather than data
    ///
    /// let mut post_order = Traversal
    ///     .post_order()
    ///     .over_nodes()           // node rather than data
    ///     .with_depth()           // => (depth, node)
    ///     .with_sibling_idx();    // => (depth, sibling_idx, node)
    ///
    /// let mut iter = post_order.iter(&n3);
    ///
    /// let (depth, sibling_idx, node) = iter.next().unwrap();
    /// assert_eq!(node.data(), &5);
    /// assert_eq!(node.num_children(), 0);
    /// assert_eq!(node.parent().map(|x| *x.data()), Some(3));
    /// assert_eq!(depth, 1);
    /// assert_eq!(sibling_idx, 0);
    ///
    /// let (depth, sibling_idx, node) = iter.next().unwrap();
    /// assert_eq!(node.data(), &6);
    /// assert_eq!(node.num_children(), 0);
    /// assert_eq!(node.parent().map(|x| *x.data()), Some(3));
    /// assert_eq!(depth, 1);
    /// assert_eq!(sibling_idx, 1);
    ///
    /// let (depth, sibling_idx, node) = iter.next().unwrap();
    /// assert_eq!(node.data(), &3);
    /// assert_eq!(node.num_children(), 2);
    /// assert_eq!(node.parent().map(|x| *x.data()), Some(2));
    /// assert_eq!(depth, 0); // as it is the root of the traversed subtree
    /// assert_eq!(sibling_idx, 0);
    /// ```
    fn iter<'a, M, P>(
        &mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        O: 'a,
        Self: 'a;

    /// Consumes this traverser and returns a transformed version of it
    /// which creates iterators over `O2` rather than `O2`.
    fn transform_into<O2: Over<V>>(self) -> Self::IntoOver<O2>;

    /// Returns the transformed version of the traverser where it yields:
    /// * data rather than [`Node`]
    /// * (depth, data) rather than (depth, [`Node`])
    /// * (depth, sibling_idx, data) rather than (depth, sibling_idx, [`Node`])
    ///
    /// [`Node`]: crate::Node
    fn over_data(self) -> Self::IntoOver<O::IntoOverData> {
        self.transform_into::<O::IntoOverData>()
    }

    /// Returns the transformed version of the traverser where it yields:
    /// * [`Node`] rather than data
    /// * (depth, [`Node`]) rather than (depth, data)
    /// * (depth, sibling_idx, [`Node`]) rather than (depth, sibling_idx, data)
    ///
    /// [`Node`]: crate::Node
    fn over_nodes(self) -> Self::IntoOver<O::IntoOverNode> {
        self.transform_into::<O::IntoOverNode>()
    }

    /// Returns the transformed version of the traverser where it yields:
    ///
    /// * (depth, x) rather than x
    /// * (depth, sibling_idx, x) rather than (sibling_idx, x)
    ///
    /// where x might data or [`Node`].
    ///
    /// [`Node`]: crate::Node
    fn with_depth(self) -> Self::IntoOver<O::IntoWithDepth> {
        self.transform_into::<O::IntoWithDepth>()
    }

    /// Returns the transformed version of the traverser where it yields:
    ///
    /// * (sibling_idx, x) rather than x
    /// * (depth, sibling_idx, x) rather than (depth, x)
    ///
    /// where x might data or [`Node`].
    ///
    /// [`Node`]: crate::Node
    fn with_sibling_idx(self) -> Self::IntoOver<O::IntoWithSiblingIdx> {
        self.transform_into::<O::IntoWithSiblingIdx>()
    }
}
