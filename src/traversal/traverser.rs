use std::dbg;

use super::{
    over::{Over, OverItem},
    over_mut::{OverItemInto, OverItemMut},
    OverMut,
};
use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage, NodeMut, NodeRef, TreeVariant};

/// A tree traverser that creates iterators which walk over a given node and all of its descendants;
/// i.e., over all nodes of the sub-tree rooted at the given node.
///
/// The order the nodes are traversed depend on the specific traverser implementation; some well known
/// traversals are depth-first, breadth-first or post-order.
///
/// A traverser holds its temporary data, and therefore, it might be used to create as many iterators
/// as needed without requiring additional allocation.
///
/// It creates three kinds of iterators with its three iterator methods.
/// Each of these methods take the root node of the traversal as its argument.
///
/// * [`iter`]
///   * Creates an iterator over references.
///   * `Iterator<Item = &V::Item>`
///   * The tree remains unchanged.
/// * [`iter_mut`]
///   * Creates an iterator over mutable references.
///   * `Iterator<Item = &mut V::Item>`
///   * The data of the subtree rooted at the given node might be mutated.
///   * However, the structure of the tree remains unchanged.
/// * [`into_iter`]
///   * Creates an iterator over owned values taken out of the nodes.
///   * `Iterator<Item = V::Item>`
///   * All nodes belonging to the subtree rooted at the given node will be removed.
///   * Corresponding data of the removed nodes will be yield in the order of the traversal.
///
/// # Construction
///
/// A traverser can be created by its `Default::default()` method such as:
///
/// ```
/// use orx_tree::{*, traversal::*};
///
/// let mut traverser = Dfs::<Dyn<i32>>::default();
/// let mut traverser = Bfs::<Binary<i32>>::default();
/// let mut traverser = PostOrder::<Dary<4, i32>>::default();
///
/// // or traverser to iterate over different items
/// let mut traverser = Dfs::<Dyn<i32>, OverNode>::default(); // yields Node rather than data
/// let mut traverser = Bfs::<Binary<i32>, OverDepthData>::default(); // yields (depth, data)
/// let mut traverser = PostOrder::<Dary<4, OverDepthSiblingIdxData>>::default(); // yields (depth, sibling_idx, data)
/// ```
///
/// However, it is often more convenient to use the [`Traversal`] type to create the traverser instances;
/// and transform them to yield different items if needed.
///
/// ```ignore
/// use orx_tree::*;
///
/// let mut traverser = Traversal.dfs();
/// let mut traverser = Traversal.bfs();
/// let mut traverser = Traversal.post_order();
///
/// // or traverser to iterate over different items
/// let mut traverser = Traversal.dfs().over_nodes(); // yields Node rather than data
/// let mut traverser = Traversal.bfs().with_depth(); // yields (depth, data)
/// let mut traverser = Traversal.post_order().with_depth().with_sibling_idx(); // yields (depth, sibling_idx, data)
/// ```
///
/// # Examples
///
/// The following example demonstrates multiple use of all three kinds of iterator generating methods by a
/// traverser.
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
/// // create a traverser once
/// // and use it multiple times without allocation
///
/// let mut traverser = Traversal.dfs();
///
/// // [I] iter: iterate over references
///
/// let root = id1.node(&tree);
/// let tree_vals: Vec<&i32> = traverser.iter(&root).collect();
/// assert_eq!(tree_vals, [&1, &2, &4, &8, &5, &3, &6, &9, &7, &10, &11]);
///
/// let n3 = id3.node(&tree);
/// let from_n3: Vec<&i32> = traverser.iter(&n3).collect();
/// assert_eq!(from_n3, [&3, &6, &9, &7, &10, &11]);
///
/// // [II] iter_mut: iterate over mutable references
///
/// let mut n7 = id7.node_mut(&mut tree);
/// for x in traverser.iter_mut(&mut n7) {
///     // must yield 10 -> 11 -> 7
///     *x += 100;
/// }
///
/// let root = id1.node(&tree);
/// let tree_vals: Vec<&i32> = traverser.iter(&root).collect();
/// assert_eq!(
///     tree_vals,
///     [&1, &2, &4, &8, &5, &3, &6, &9, &107, &110, &111]
/// );
///
/// // [III] into_iter: iterate over removed values
///
/// let n3 = id3.node_mut(&mut tree);
/// let removed: Vec<i32> = traverser.into_iter(n3).collect();
/// assert_eq!(removed, [3, 6, 9, 107, 110, 111]);
///
/// // all 6 nodes are removed from the tree
/// let root = id1.node(&tree);
/// let tree_vals: Vec<&i32> = traverser.iter(&root).collect();
/// assert_eq!(tree_vals, [&1, &2, &4, &8, &5]); // remaining nodes
///
/// // let's completely drain the tree: into_iter(root)
/// let root = id1.node_mut(&mut tree);
/// let removed: Vec<i32> = traverser.into_iter(root).collect();
/// assert_eq!(removed, [1, 2, 4, 8, 5]);
/// assert!(tree.is_empty());
/// assert_eq!(tree.root(), None);
/// ```
///
/// # Iterating Over Different Values
///
/// For [`iter`], it is possible to iterate over [`Node`]s rather than node data.
///
/// Further, for all three iterator methods, it is possible to add either or both of:
///
/// * **depth** of the traversed node,
/// * **sibling_idx** of the traversed node among its siblings
///
/// to node value which is either node data or the node itself.
///
/// The return value of the iterations depend on the second generic parameter of the traverser which implements
/// the [`Over`] trait. The following is the complete list of implementations and the corresponding item type
/// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
/// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
///
/// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
/// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
///
/// Further, **D** refers to node data, which is:
/// * `&V::Item` with `iter`,
/// * `&mut V::Item` with `iter_mut`, and
/// * `V::Item` with `into_iter`.
///
/// | Over | Yields | Depth First Example |
/// |---|---|---|
/// | [`OverData`] | D | `Traversal.dfs()` |
/// | [`OverDepthData`] | (depth, D) | `Traversal.dfs().with_depth()` |
/// | [`OverSiblingIdxData`] | (sibling_idx, D) | `Traversal.dfs().with_sibling_idx()` |
/// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, D) | `Traversal.with_depth().with_sibling_idx()` |
/// | [`OverNode`] | Node | `Traversal.dfs().over_nodes()` |
/// | [`OverDepthNode`] | (depth, Node) | `Traversal.dfs().over_nodes().with_depth()` |
/// | [`OverSiblingIdxNode`] | (sibling_idx, Node) | `Traversal.dfs().over_nodes().with_sibling_idx()` |
/// | [`OverDepthSiblingIdxNode`] | (depth, sibling_idx, Node) | `Traversal.dfs().over_nodes().with_depth().with_sibling_idx()` |
///
/// [`iter`]: crate::traversal::Traverser::iter
/// [`iter_mut`]: crate::traversal::Traverser::iter_mut
/// [`into_iter`]: crate::traversal::Traverser::into_iter
/// [`Node`]: crate::Node
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
        M: MemoryPolicy,
        P: PinnedStorage,
        O: 'a,
        Self: 'a;

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
        O: OverMut<V> + 'a,
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
        O: OverMut<V> + 'a,
        Self: 'a,
    {
        core::iter::empty()
    }

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
