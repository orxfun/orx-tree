use super::{BfsIterable, DfsIterable, IterOver, OverData};
use crate::TreeVariant;

/// Type to create iterables which are capable of repeatedly creating iterators
/// corresponding to different kinds of traversals starting from different tree nodes
/// without allocating.
///
/// # Traversals
///
/// Currently, it is capable of creating iterables for three kinds of traversals.
///
/// ## dfs (depth first search)
///
///  * dfs iteration internally uses a stack (alloc::vec::Vec).
/// * A dfs iterator can be created from a tree node using [`dfs`], [`dfs_mut`],
///   [`dfs_over`] or [`dfs_mut_over`]. Each time these methods are called a
///   stack will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a dfs iterable using [`TreeIter::dfs`] or
///   [`TreeIter::dfs_over`] methods. These iterables allocate the stack only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// ## bfs (breadth first search)
///
/// * bfs iteration internally uses a queue (alloc::collections::VecDeque).
/// * A bfs iterator can be created from a tree node using [`bfs`], [`bfs_mut`],
///   [`bfs_over`] or [`bfs_mut_over`]. Each time these methods are called a
///   queue will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a bfs iterable using [`TreeIter::bfs`] or
///   [`TreeIter::bfs_over`] methods. These iterables allocate the queue only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// ## post-order
///
///  * post order iteration internally uses a vector (alloc::vec::Vec) of length **D**
///   where D is the maximum depth of nodes visited throughout the traversal.
/// * A post order iterator can be created from a tree node using [`post_order`], [`post_order_mut`],
///   [`post_order_over`] or [`post_order_mut_over`]. Each time these methods are called a
///   vector will be created and dropped at the end of the iteration.
/// * Alternatively, one can create a post order iterable using [`TreeIter::post_order`] or
///   [`TreeIter::post_order_over`] methods. These iterables allocate the vector only
///   once and can be used to repeatedly create iterators without new allocations.
///
/// [`dfs`]: crate::NodeRef::dfs
/// [`dfs_mut`]: crate::NodeMut::dfs_mut
/// [`dfs_over`]: crate::NodeRef::dfs_over
/// [`dfs_mut_over`]: crate::NodeMut::dfs_mut_over
/// [`bfs`]: crate::NodeRef::bfs
/// [`bfs_mut`]: crate::NodeMut::bfs_mut
/// [`bfs_over`]: crate::NodeRef::bfs_over
/// [`bfs_mut_over`]: crate::NodeMut::bfs_mut_over
/// [`post_order`]: crate::NodeRef::post_order
/// [`post_order_mut`]: crate::NodeMut::post_order_mut
/// [`post_order_over`]: crate::NodeRef::post_order_over
/// [`post_order_mut_over`]: crate::NodeMut::post_order_mut_over
pub struct Traversals;

impl Traversals {
    /// Creates a depth-first-search iterable internally using a stack (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    /// See also [`dfs_over`] for other variants.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`dfs_over`]: Self::dfs_over
    /// [`OverData`]: crate::iter::OverData
    pub fn dfs<V: TreeVariant>() -> DfsIterable<V, OverData> {
        Default::default()
    }

    /// Creates a depth-first-search iterable internally using a stack (alloc::vec::Vec).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse over values defined by the generic parameter [`O`]:
    ///
    /// * [`OverData`] yields data of nodes
    /// * [`OverDepthData`] yields (depth, data) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    pub fn dfs_over<V: TreeVariant, O: IterOver>() -> DfsIterable<V, O> {
        Default::default()
    }

    /// Creates a breadth-first-search iterable internally using a queue (alloc::collections::VecDeque).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse [`OverData`]; i.e., yield a reference or a mutable
    /// reference to the data of the nodes.
    /// See also [`bfs_over`] for other variants.
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`bfs_over`]: Self::bfs_over
    /// [`OverData`]: crate::iter::OverData
    pub fn bfs<V: TreeVariant>() -> BfsIterable<V, OverData> {
        Default::default()
    }

    /// Creates a breadth-first-search iterable internally using a queue (alloc::collections::VecDeque).
    ///
    /// Created iterable can be used repeatedly without allocation to create iterators
    /// using [`iter`] method and mutable iterators using [`iter_mut`] starting from any
    /// node of a tree.
    ///
    /// Created iterators traverse over values defined by the generic parameter [`O`]:
    ///
    /// * [`OverData`] yields data of nodes
    /// * [`OverDepthData`] yields (depth, data) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data) tuples where the second element is a usize representing the index of the node among its siblings
    /// * [`OverNode`] yields directly the nodes ([`Node`])
    /// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
    /// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
    ///
    /// [`iter`]: crate::iter::DfsIterable::iter
    /// [`iter_mut`]: crate::iter::DfsIterable::iter_mut
    /// [`OverData`]: crate::iter::OverData
    /// [`OverDepthData`]: crate::iter::OverDepthData
    /// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
    /// [`OverNode`]: crate::iter::OverNode
    /// [`OverDepthNode`]: crate::iter::OverDepthNode
    /// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
    pub fn bfs_over<V: TreeVariant, O: IterOver>() -> BfsIterable<V, O> {
        Default::default()
    }
}
