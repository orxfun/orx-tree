use crate::{pinned_storage::PinnedStorage, Tree, TreeVariant};
use orx_selfref_col::{MemoryPolicy, MemoryReclaimNever, MemoryReclaimOnThreshold};

pub trait TreeMemoryPolicy: 'static {
    type MemoryReclaimPolicy<V>: MemoryPolicy<V>
    where
        V: TreeVariant;
}

/// Trees use a pinned vector ([`PinnedVec`]) as its underlying storage.
/// Deletions from the tree are lazy in the sense that the nodes are closed; not removed from the vector.
///
/// The `Lazy` policy never reclaims closed nodes.
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
/// [`PinnedVec`]: orx_pinned_vec::PinnedVec
/// [`NodeIdx<V>`]: orx_selfref_col::NodeIdx
pub struct Lazy;
impl TreeMemoryPolicy for Lazy {
    type MemoryReclaimPolicy<V>
        = MemoryReclaimNever
    where
        V: TreeVariant;
}

/// Trees use a pinned vector ([`PinnedVec`]) as its underlying storage.
/// Deletions from the tree are lazy in the sense that the nodes are closed; not removed from the vector.
///
/// The `Auto` policy reclaims closed nodes whenever the utilization falls below 75%.
/// Since utilization can only drop on removals, only removals can trigger a memory reclaim.
/// Growth methods such as push, extend or grow will never trigger a memory reclaim.
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
///
/// [`PinnedVec`]: orx_pinned_vec::PinnedVec
/// [`NodeIdx<V>`]: orx_selfref_col::NodeIdx
///
/// # Examples
///
/// Since it is the default memory policy, we do not need to include it in the tree type signature.
/// The following example demonstrates how this policy impacts the validity of node indices.
/// Please also see the example of the [`Lazy`] policy.
/// Further, the second example will demonstrate how to switch between the policies to provide the
/// means to use both node indices and memory efficiently.
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
///     tree.root().unwrap().bfs().copied().collect()
/// }
///
/// // # 1. GROW
///
/// let mut tree = DynTree::<i32>::new(1); // or => DynTree::<i32, Auto>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
/// let [id2, id3] = root.grow([2, 3]);
///
/// let mut n2 = id2.node_mut(&mut tree);
/// let [id4, _] = n2.grow([4, 5]);
///
/// let [id8] = id4.node_mut(&mut tree).grow([8]);
///
/// let mut n3 = id3.node_mut(&mut tree);
/// let [id6, id7] = n3.grow([6, 7]);
///
/// id6.node_mut(&mut tree).push(9);
/// id7.node_mut(&mut tree).extend([10, 11]);
///
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
///
/// // since the tree only grew, we know that all id's above are valid
/// assert!(id2.is_valid_for(&tree)); // is_valid_for => true
/// assert!(id4.get_node(&tree).is_some()); // get_node => Some(Node)
/// assert!(id6.try_get_node(&tree).is_ok()); // try_get_node => Ok(Node)
/// let _node7 = id7.node(&tree); // no panic
///
/// // # 2 - SHRINK BUT WITHOUT A MEMORY RECLAIM
///
/// // let's close two nodes (nodes 4 & 8)
/// // this is not enough to trigger a memory reclaim
/// id4.node_mut(&mut tree).remove();
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 7, 9, 10, 11]);
///
/// assert!(id2.is_valid_for(&tree)); // is_valid_for => true
/// assert!(id6.try_get_node(&tree).is_ok()); // try_get_node => Ok(Node)
/// let node7 = id7.node(&tree); // no panic
///
/// // what about id4 & id8 => invalidated due to RemovedNode
/// assert!(!id4.is_valid_for(&tree));
/// assert!(id4.get_node(&tree).is_none());
/// assert_eq!(id4.try_get_node(&tree), Err(NodeIdxError::RemovedNode));
/// // let node4 = id4.node(&tree); // panics!!!
///
/// // # 3 - SHRINK TRIGGERING MEMORY RECLAIM
///
/// // let's close more nodes (7, 10, 11) to trigger the memory reclaim
/// id7.node_mut(&mut tree).remove();
/// assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 9]);
///
/// // even node 2 is still on the tree;
/// // its idx is invalidated => ReorganizedCollection
/// assert!(!id2.is_valid_for(&tree));
/// assert_eq!(
///     id2.try_get_node(&tree),
///     Err(NodeIdxError::ReorganizedCollection)
/// );
///
/// // all indices obtained prior to reorganization are now invalid
/// // we can restore the valid indices again
///
/// let id2 = tree.root().unwrap().child(0).unwrap().idx();
/// assert!(id2.is_valid_for(&tree));
/// assert!(id2.try_get_node(&tree).is_ok());
/// let n2 = id2.node(&tree);
/// assert_eq!(n2.data(), &2);
/// ```
///
/// Now assume that we want to make sure that the indices we cached during growth
/// are valid during the stages #2 and #3.
/// We can achieve this by switching to Lazy policy and only after stage #3,
/// when we are done using the indices, we can switch back to Auto policy.
/// Note that these transformations of the memory policies are free.
///
///
pub struct Auto;
impl TreeMemoryPolicy for Auto {
    type MemoryReclaimPolicy<V>
        = MemoryReclaimOnThreshold<2, V, V::Reclaimer>
    where
        V: TreeVariant;
}

pub struct AutoWithThreshold<const D: usize>;
impl<const D: usize> TreeMemoryPolicy for AutoWithThreshold<D> {
    type MemoryReclaimPolicy<V>
        = MemoryReclaimOnThreshold<D, V, V::Reclaimer>
    where
        V: TreeVariant;
}

// tree memory policy transformations: into_lazy_reclaim

impl<V, P> Tree<V, Auto, P>
where
    V: TreeVariant,
    P: PinnedStorage,
{
    /// Transforms the tree's memory reclaim policy from [`Auto`] to [`Lazy`]; i.e.,
    /// `Tree<V, Auto>` becomes `Tree<V, Lazy>`.
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
    pub fn into_auto_reclaim(self) -> Tree<V, Auto, P> {
        Tree(self.0.into())
    }

    pub fn into_auto_reclaim_with_threshold<const D: usize>(
        self,
    ) -> Tree<V, AutoWithThreshold<D>, P> {
        Tree(self.0.into())
    }
}

#[test]
fn abc() {
    use crate::*;
    use alloc::vec::Vec;

    // # 1 - BUILD UP

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11

    fn bfs_values<M: TreeMemoryPolicy>(tree: &DynTree<i32, M>) -> Vec<i32> {
        tree.root().unwrap().bfs().copied().collect()
    }

    // # 1. GROW

    let mut tree = DynTree::<i32, Auto>::new(1);
    // let tree: DynTree<i32, Lazy> = tree.into_lazy_reclaim();

    let mut root = tree.root_mut().unwrap();
    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    let [id4, _] = n2.grow([4, 5]);

    let [id8] = id4.node_mut(&mut tree).grow([8]);

    let mut n3 = id3.node_mut(&mut tree);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree).push(9);
    id7.node_mut(&mut tree).extend([10, 11]);

    assert_eq!(bfs_values(&tree), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    // since the tree only grew, we know that all id's above are valid
    assert!(id2.is_valid_for(&tree)); // is_valid_for => true
    assert!(id4.get_node(&tree).is_some()); // get_node => Some(Node)
    assert!(id6.try_get_node(&tree).is_ok()); // try_get_node => Ok(Node)
    let _node7 = id7.node(&tree); // no panic!

    // # 2 - SHRINK BUT WITHOUT A MEMORY RECLAIM

    // let's close two nodes (nodes 4 & 8)
    // this is not enough to trigger a memory reclaim
    id4.node_mut(&mut tree).remove();
    assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 7, 9, 10, 11]);

    assert!(id2.is_valid_for(&tree)); // is_valid_for => true
    assert!(id6.try_get_node(&tree).is_ok()); // try_get_node => Ok(Node)
    let node7 = id7.node(&tree); // no panic

    // what about id4 & id8 => invalidated due to RemovedNode
    assert!(!id4.is_valid_for(&tree));
    assert!(id4.get_node(&tree).is_none());
    assert_eq!(id4.try_get_node(&tree), Err(NodeIdxError::RemovedNode));
    // let node4 = id4.node(&tree); // panics!

    // # 3 - SHRINK TRIGGERING MEMORY RECLAIM

    // let's close more nodes (7, 10, 11) to trigger the memory reclaim
    id7.node_mut(&mut tree).remove();
    assert_eq!(bfs_values(&tree), [1, 2, 3, 5, 6, 9]);

    // even node 2 is still on the tree;
    // its idx is invalidated => ReorganizedCollection
    assert!(!id2.is_valid_for(&tree));
    assert_eq!(
        id2.try_get_node(&tree),
        Err(NodeIdxError::ReorganizedCollection)
    );

    // all indices obtained prior to reorganization are now invalid
    // we can restore the valid indices again

    let id2 = tree.root().unwrap().child(0).unwrap().idx();
    assert!(id2.is_valid_for(&tree));
    assert!(id2.try_get_node(&tree).is_ok());
    let n2 = id2.node(&tree);
    assert_eq!(n2.data(), &2);
}
