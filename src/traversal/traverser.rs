use super::{
    over::{Over, OverItem},
    over_mut::{OverItemInto, OverItemMut},
    traverser_core::TraverserCore,
    OverData, OverMut,
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
/// A traverser can be created by its `new` or `default` method such as:
///
/// ```
/// use orx_tree::{*, traversal::*};
///
/// let mut traverser = Dfs::default();
/// let mut traverser = Bfs::default();
/// let mut traverser = PostOrder::default();
///
/// // or traverser to iterate over different items
/// let mut traverser = Dfs::<OverNode>::new(); // yields Node rather than data
/// let mut traverser = Bfs::<OverDepthData>::new(); // yields (depth, data)
/// let mut traverser = PostOrder::<OverDepthSiblingIdxData>::new(); // yields (depth, sibling_idx, data)
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
pub trait Traverser<O = OverData>: TraverserCore<O>
where
    O: Over,
{
    /// Transformed version of the traverser from creating iterators over `O` to `O2`.
    type IntoOver<O2>
    where
        O2: Over;

    /// Creates a new traverser.
    fn new() -> Self;

    // fn iter<'a, V, M, P>(
    //     &'a mut self,
    //     node: &'a impl NodeRef<'a, V, M, P>,
    // ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    // where
    //     V: TreeVariant + 'a,
    //     M: MemoryPolicy,
    //     P: PinnedStorage;

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
    fn iter_mut<'a, V, M, P>(
        &'a mut self,
        node: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut;

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
    #[allow(clippy::wrong_self_convention)]
    fn into_iter<'a, V, M, P>(
        &'a mut self,
        node: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut;

    /// Consumes this traverser and returns a transformed version of it
    /// which creates iterators over `O2` rather than `O2`.
    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2>;

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
