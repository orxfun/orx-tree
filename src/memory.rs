use crate::{Tree, TreeVariant, pinned_storage::PinnedStorage};
use orx_selfref_col::{MemoryReclaimNever, MemoryReclaimOnThreshold, MemoryReclaimer, Utilization};

/// Trees use a pinned vector ([`PinnedVec`]) as its underlying storage.
/// Deletions from the tree are lazy in the sense that the nodes are closed; not removed from the vector.
/// Therefore, deletions leave a gap in the underlying collection.
/// How these nodes will be reclaimed is determined by the `MemoryPolicy`.
///
/// This is important because during memory reclaim, nodes of the collection are reorganized which
/// invalidates the node indices which are obtained prior to this operation.
/// Node indices, or [`NodeIdx`], are analogous to usize indices of a slice and allow for safe direct
/// constant time access to the node it is created for.
///
/// There are three available policies:
///
/// * [`Auto`]
///   * The default policy.
///   * It automatically reclaims memory whenever the utilization falls below 75%.
///   * Automatic triggers of memory reclaims might lead to implicit invalidation of node indices due to [`ReorganizedCollection`].
///   * It is important to note that, even when using Auto policy, tree growth will never trigger memory reclaim.
///     Only node removing mutations such as [`prune`] can trigger node reorganization.
/// * [`AutoWithThreshold`]
///   * This is a generalization of the Auto policy; in particular, Auto is equivalent to `AutoWithThreshold<2>`.
///   * Its constant parameter `D` defines the utilization threshold to trigger the memory reclaim operation.
///   * Specifically, memory of closed nodes will be reclaimed whenever the ratio of closed nodes to all nodes exceeds one over `2^D`.
///     The greater the `D`, the greater the utilization threshold.
/// * [`Lazy`]
///   * It never reclaims memory.
///   * It never leads to implicit invalidation of node indices.
///     In other words, a node index can only be invalidated if we prune that node from the tree ([`RemovedNode`]).
///
/// An ideal use pattern can conveniently be achieved by using auto and lazy policies together,
/// using the free memory policy transformation methods.
/// Such a use case is demonstrated in the example below.
///
/// [`PinnedVec`]: orx_pinned_vec::PinnedVec
/// [`NodeIdx`]: crate::NodeIdx
/// [`prune`]: crate::NodeMut::prune
/// [`ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
/// [`RemovedNode`]: crate::NodeIdxError::RemovedNode
///
/// # Examples
///
/// ## Auto Memory Claim
///
/// Since Auto is the default memory policy, we do not need to include it in the tree type signature.
/// The following example demonstrates how this policy impacts the validity of node indices.
///
/// The follow up example will demonstrate how to switch between the [`Lazy`] and [`Auto`] policies
/// to make best use of these policies and use both node indices and memory efficiently.
///
/// ```
/// use orx_tree::*;
///
/// // # 1 - BUILD UP
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
/// fn bfs_values(tree: &DynTree<i32>) -> Vec<i32> {
///     tree.root().walk::<Bfs>().copied().collect()
/// }
///
/// // # 1. GROW
///
/// // equivalently => DynTree::<i32, Auto>::new(1)
/// let mut tree = DynTree::new(1);
///
/// let [id2, id3] = tree.root_mut().push_children([2, 3]);
/// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
/// let [id8] = tree.node_mut(id4).push_children([8]);
/// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
/// tree.node_mut(id6).push_child(9);
/// tree.node_mut(id7).push_children([10, 11]);
///
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
///
/// // all id's above are valid => the tree only grew
/// assert!(tree.is_node_idx_valid(&id2)); // is_valid_for => true
/// assert!(tree.get_node(id4).is_some()); // get_node => Some(Node)
/// assert!(tree.try_node(&id6).is_ok()); // try_get_node => Ok(Node)
/// let _node7 = tree.node(id7); // no panic
///
/// // # 2 - SHRINK BUT WITHOUT A MEMORY RECLAIM
///
/// // let's close two nodes (nodes 4 & 8)
/// // this is not enough to trigger a memory reclaim
/// tree.node_mut(id4).prune();
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 7, 9, 10, 11]);
///
/// assert!(tree.is_node_idx_valid(&id2)); // is_valid_for => true
/// assert!(tree.try_node(&id6).is_ok()); // try_get_node => Ok(Node)
/// let node7 = tree.node(id7); // no panic
///
/// // what about id4 & id8 => invalidated due to RemovedNode
/// assert!(!tree.is_node_idx_valid(&id4));
/// assert!(tree.get_node(id4).is_none());
/// assert_eq!(tree.try_node(&id4), Err(NodeIdxError::RemovedNode));
/// // let node4 = id4.node(tree); // panics!!!
///
/// // # 3 - SHRINK TRIGGERING MEMORY RECLAIM
///
/// // let's close more nodes (7, 10, 11) to trigger the memory reclaim
/// tree.node_mut(id7).prune();
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 9]);
///
/// // even node 2 is still on the tree;
/// // its idx is invalidated => ReorganizedCollection
/// assert!(!tree.is_node_idx_valid(&id2));
/// assert_eq!(
///     tree.try_node(&id2),
///     Err(NodeIdxError::ReorganizedCollection)
/// );
///
/// // all indices obtained prior to reorganization are now invalid
/// // we can restore the valid indices again
///
/// let id2 = tree.root().get_child(0).unwrap().idx();
/// assert!(tree.is_node_idx_valid(&id2));
/// assert!(tree.try_node(&id2).is_ok());
/// let n2 = tree.node(id2);
/// assert_eq!(n2.data(), &2);
/// ```
///
/// ## Lazy Memory Claim: Preventing Invalid Indices
///
/// Now assume that we want to make sure that the indices we cached during growth
/// are valid during the stages #2 and #3.
/// We can achieve this by switching to Lazy policy and only after stage #3,
/// when we are done using the indices, we can switch back to Auto policy.
/// Note that these transformations of the memory policies are free.
///
/// ```
/// use orx_tree::*;
///
/// // # 1 - BUILD UP
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
/// fn bfs_values<M: MemoryPolicy>(tree: &DynTree<i32, M>) -> Vec<i32> {
///     tree.root().walk::<Bfs>().copied().collect()
/// }
///
/// // # 1. GROW
///
/// // or just => DynTree::<i32, Lazy>::new(1);
/// let mut tree = DynTree::new(1).into_lazy_reclaim();
///
/// let [id2, id3] = tree.root_mut().push_children([2, 3]);
/// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
/// let [id8] = tree.node_mut(id4).push_children([8]);
/// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
/// tree.node_mut(id6).push_child(9);
/// tree.node_mut(id7).push_children([10, 11]);
///
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
///
/// // all id's above are valid => we are in Lazy mode & the tree only grew
/// assert!(tree.is_node_idx_valid(&id2)); // is_valid_for => true
/// assert!(tree.get_node(id4).is_some()); // get_node => Some(Node)
/// assert!(tree.try_node(&id6).is_ok()); // try_get_node => Ok(Node)
/// let _node7 = tree.node(id7); // no panic!
///
/// // # 2 - SHRINK, NO MEMORY RECLAIM
///
/// // let's close two nodes (nodes 4 & 8)
/// tree.node_mut(id4).prune();
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 7, 9, 10, 11]);
///
/// assert!(tree.is_node_idx_valid(&id2)); // is_valid_for => true
/// assert!(tree.try_node(&id6).is_ok()); // try_get_node => Ok(Node)
/// let node7 = tree.node(id7); // no panic
///
/// // only id4 & id8 are affected (explicit) => invalidated due to RemovedNode
/// assert!(!tree.is_node_idx_valid(&id4));
/// assert!(tree.get_node(id4).is_none());
/// assert_eq!(tree.try_node(&id4), Err(NodeIdxError::RemovedNode));
/// // let node4 = id4.node(tree); // panics!
///
/// // # 3 - SHRINK HEAVILY, STILL NO MEMORY RECLAIM
///
/// // let's close more nodes (7, 10, 11)
/// // this would've triggered memory reclaim in Auto policy, but not in Lazy policy
/// tree.node_mut(id7).prune();
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 9]);
///
/// // all indices are still valid ✓
/// assert!(tree.is_node_idx_valid(&id2));
/// assert!(tree.try_node(&id2).is_ok());
/// let n2 = tree.node(id2);
/// assert_eq!(n2.data(), &2);
///
/// // # 4 - END OF HEAVY MUTATIONS, RECLAIM THE MEMORY
///
/// // since utilization was below 75% (6 active nodes / 11)
/// // memory is reclaimed immediately once switch to Auto.
/// // now, all prior indices are invalid
/// let tree: DynTree<i32, Auto> = tree.into_auto_reclaim();
/// assert!(!tree.is_node_idx_valid(&id2));
/// assert!(tree.get_node(id3).is_none());
/// assert_eq!(
///     tree.try_node(&id4),
///     Err(NodeIdxError::ReorganizedCollection)
/// );
/// ```
pub trait MemoryPolicy: 'static {
    /// Memory reclaim policy for the specific tree variant `V`.
    type MemoryReclaimPolicy<V>: orx_selfref_col::MemoryPolicy<V>
    where
        V: TreeVariant;
}

/// The `Lazy` policy never reclaims closed nodes.
///
/// * It never reclaims memory.
/// * It never leads to implicit invalidation of node indices.
///   In other words, a node index can only be invalidated if we remove that node from the tree ([`RemovedNode`]).
///
/// Compared to `Auto`, lazy approach has the following pros & cons:
///
/// * (+) Node indices ([`NodeIdx<V>`]) created for a lazy tree will never be implicitly invalidated.
///   Recall that, analogous to random access to an array element with its index,
///   node indices allow for constant time access to the corresponding node.
///   A node index can be invalidated if either of the following happens
///   (i) the node is removed from the tree,
///   (ii) other nodes are removed from the tree triggering a memory reclaim operation.
///   With lazy policy, the second case can never happen.
/// * (-) Uses more memory due to the gaps of the closed nodes especially when there exist many deletions
///   from the tree.
///
/// Notice that the con can never be observed for grow-only situations; or might be insignificant when the
/// number of removals is not very large. In such situations, it is the recommended memory reclaim policy.
///
/// [`NodeIdx<V>`]: orx_selfref_col::NodeIdx
/// [`RemovedNode`]: crate::NodeIdxError::RemovedNode
pub struct Lazy;
impl MemoryPolicy for Lazy {
    type MemoryReclaimPolicy<V>
        = MemoryReclaimNever
    where
        V: TreeVariant;
}

/// The `Auto` policy reclaims closed nodes whenever the utilization falls below 75%.
///
/// * The default policy; and hence, the generic parameter can be omitted.
/// * It automatically reclaims memory whenever the utilization falls below 75%.
/// * Automatic triggers of memory reclaims might lead to implicit invalidation of node indices due to [`ReorganizedCollection`].
/// * It is important to note that, even when using Auto policy, tree growth will never trigger memory reclaim.
///   Only node removing mutations such as [`prune`] can trigger node reorganization.
///
/// Compared to `Lazy`, auto approach has the following pros & cons:
///
/// * (+) Node indices ([`NodeIdx<V>`]) created for an auto-reclaim tree will can be implicitly invalidated.
///   Recall that, analogous to random access to an array element with its index,
///   node indices allow for constant time access to the corresponding node.
///   A node index can be invalidated if either of the following happens
///   (i) the node is removed from the tree,
///   (ii) other nodes are removed from the tree triggering a memory reclaim operation.
///   With auto policy, both the explicit (i) and implicit (ii) invalidation can occur.
/// * (-) Uses memory efficiently making sure that utilization can never go below a constant threshold.
///
/// [`NodeIdx<V>`]: orx_selfref_col::NodeIdx
///
/// Since Auto is the default memory policy, we do not need to include it in the tree type signature.
///
/// In order to observe the impact of the memory reclaim policy on validity of the node indices,
/// please see the Examples section of [`MemoryPolicy`].
///
/// [`ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
/// [`prune`]: crate::NodeMut::prune
pub struct Auto;
impl MemoryPolicy for Auto {
    type MemoryReclaimPolicy<V>
        = MemoryReclaimOnThreshold<2, V, V::Reclaimer>
    where
        V: TreeVariant;
}

/// The `AutoWithThreshold` policy reclaims closed nodes whenever the utilization falls below a certain threshold
/// which is determined by the constant parameter `D`.
///
/// * This is a generalization of the [`Auto`] policy; in particular, Auto is equivalent to `AutoWithThreshold<2>`.
/// * Its constant parameter `D` defines the utilization threshold to trigger the memory reclaim operation.
/// * Specifically, memory of closed nodes will be reclaimed whenever the ratio of closed nodes to all nodes exceeds one over `2^D`.
///   The greater the `D`, the greater the utilization threshold.
///   * when `D = 0`: memory will be reclaimed when utilization is below 0.00% (equivalent to never).
///   * when `D = 1`: memory will be reclaimed when utilization is below 50.00%.
///   * when `D = 2`: memory will be reclaimed when utilization is below 75.00%.
///   * when `D = 3`: memory will be reclaimed when utilization is below 87.50%.
///   * when `D = 4`: memory will be reclaimed when utilization is below 93.75%.
///
/// Compared to `Lazy`, auto approach has the following pros & cons:
///
/// * (+) Node indices ([`NodeIdx<V>`]) created for an auto-reclaim tree will can be implicitly invalidated.
///   Recall that, analogous to random access to an array element with its index,
///   node indices allow for constant time access to the corresponding node.
///   A node index can be invalidated if either of the following happens
///   (i) the node is removed from the tree,
///   (ii) other nodes are removed from the tree triggering a memory reclaim operation.
///   With auto policy, both the explicit (i) and implicit (ii) invalidation can occur.
/// * (-) Uses memory efficiently making sure that utilization can never go below a constant threshold.
///
/// [`NodeIdx<V>`]: orx_selfref_col::NodeIdx
///
/// Since Auto is the default memory policy, we do not need to include it in the tree type signature.
///
/// In order to observe the impact of the memory reclaim policy on validity of the node indices,
/// please see the Examples section of [`MemoryPolicy`].
pub struct AutoWithThreshold<const D: usize>;
impl<const D: usize> MemoryPolicy for AutoWithThreshold<D> {
    type MemoryReclaimPolicy<V>
        = MemoryReclaimOnThreshold<D, V, V::Reclaimer>
    where
        V: TreeVariant;
}

// tree methods

impl<V, M, P> Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    /// Returns current node utilization of the collection.
    pub fn memory_utilization(&self) -> Utilization {
        self.0.utilization()
    }
}

// tree memory policy transformations: into_lazy_reclaim

impl<V, P> Tree<V, Auto, P>
where
    V: TreeVariant,
    P: PinnedStorage,
{
    /// Transforms the tree's memory reclaim policy from [`Auto`] to [`Lazy`]:
    ///
    /// * `Tree<V>` => `Tree<V, Lazy>` // Auto can be omitted on since it is the default
    /// * `DynTree<u62>` => `DynTree<u32, Lazy>`
    /// * `BinaryTree<u62>` => `BinaryTree<u32, Lazy>`
    ///
    /// This is free operation and does not require any allocation or copies.
    pub fn into_lazy_reclaim(self) -> Tree<V, Lazy, P> {
        Tree(self.0.into())
    }
}

impl<const D: usize, V, P> Tree<V, AutoWithThreshold<D>, P>
where
    V: TreeVariant,
    P: PinnedStorage,
{
    /// Transforms the tree's memory reclaim policy from [`AutoWithThreshold`] to [`Lazy`]:
    ///
    /// * `Tree<V, AutoWithThreshold<3>>` => `Tree<V, Lazy>`
    /// * `DynTree<u62, AutoWithThreshold<4>>` => `DynTree<u32, Lazy>`
    /// * `BinaryTree<u62, AutoWithThreshold<1>>` => `BinaryTree<u32, Lazy>`
    ///
    /// This is free operation and does not require any allocation or copies.
    pub fn into_lazy_reclaim(self) -> Tree<V, Lazy, P> {
        Tree(self.0.into())
    }
}

// tree memory policy transformations: into_auto_reclaim

impl<V, P> Tree<V, Lazy, P>
where
    V: TreeVariant,
    P: PinnedStorage,
{
    /// Transforms the tree's memory reclaim policy from [`Lazy`] to [`Auto`]:
    ///
    /// * `Tree<V, Lazy>` => `Tree<V>` // Auto can be omitted on since it is the default
    /// * `DynTree<u62, Lazy>` => `DynTree<u32>`
    /// * `BinaryTree<u62, Lazy>` => `BinaryTree<u32>`
    ///
    /// This is free operation and does not require any allocation or copies.
    pub fn into_auto_reclaim(self) -> Tree<V, Auto, P> {
        let mut tree = Tree(self.0.into());
        let will_reclaim =
            <Auto as MemoryPolicy>::MemoryReclaimPolicy::<V>::col_needs_memory_reclaim(&tree.0);

        if will_reclaim {
            let nodes_moved = <V::Reclaimer as MemoryReclaimer<V>>::reclaim_nodes(&mut tree.0);
            tree.0.update_state(nodes_moved);
        }
        tree
    }

    /// Transforms the tree's memory reclaim policy from [`Lazy`] to [`AutoWithThreshold`]:
    ///
    /// * `Tree<V, Lazy>` => `Tree<V, AutoWithThreshold<3>>`
    /// * `DynTree<u62, Lazy>` => `DynTree<u32, AutoWithThreshold<1>>`
    /// * `BinaryTree<u62, Lazy>` => `BinaryTree<u32, AutoWithThreshold<4>>`
    ///
    /// This is free operation and does not require any allocation or copies.
    pub fn into_auto_reclaim_with_threshold<const D: usize>(
        self,
    ) -> Tree<V, AutoWithThreshold<D>, P> {
        let mut tree = Tree(self.0.into());
        let will_reclaim =
            <AutoWithThreshold<D> as MemoryPolicy>::MemoryReclaimPolicy::<V>::col_needs_memory_reclaim(&tree.0);
        if will_reclaim {
            <V::Reclaimer as MemoryReclaimer<V>>::reclaim_nodes(&mut tree.0);
        }
        tree
    }
}
