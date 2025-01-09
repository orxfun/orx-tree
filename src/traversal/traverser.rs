use super::{over::Over, traverser_core::TraverserCore, OverData};

/// A tree traverser that creates iterators which walk over a given node and all of its descendants;
/// i.e., over all nodes of the sub-tree rooted at the given node.
///
/// The order the nodes are traversed depend on the specific traverser implementation; some well known
/// traversals are depth-first, breadth-first or post-order.
///
/// A traverser holds its temporary data, and therefore, it might be used to create as many iterators
/// as needed without requiring additional allocation.
///
/// The following three kinds of node methods can be called with a traverser.
///
/// * [`walk_with`]
///   * Creates an iterator over references.
///   * `Iterator<Item = &V::Item>`
///   * The tree remains unchanged.
/// * [`walk_mut_with`]
///   * Creates an iterator over mutable references.
///   * `Iterator<Item = &mut V::Item>`
///   * The data of the subtree rooted at the given node might be mutated.
///   * However, the structure of the tree remains unchanged.
/// * [`into_walk_with`]
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
/// For immutable walks, it is possible to iterate over [`Node`]s rather than node data.
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
/// [`walk_with`]: crate::NodeRef::walk_with
/// [`walk_mut_with`]: crate::NodeMut::walk_mut_with
/// [`into_walk_with`]: crate::NodeMut::into_walk_with
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
