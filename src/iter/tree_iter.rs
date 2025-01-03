use super::{BfsIterable, DfsIterable, IterOver, OverData};
use crate::TreeVariant;

/// Factory to create iterables which are capable of repeatedly creating iterators
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
pub struct TreeIter;

impl TreeIter {
    pub fn dfs<V: TreeVariant>() -> DfsIterable<V, OverData> {
        Default::default()
    }

    pub fn dfs_over<V: TreeVariant, O: IterOver>() -> DfsIterable<V, O> {
        Default::default()
    }

    pub fn bfs<V: TreeVariant>() -> BfsIterable<V, OverData> {
        Default::default()
    }

    pub fn bfs_over<V: TreeVariant, O: IterOver>() -> BfsIterable<V, O> {
        Default::default()
    }
}
