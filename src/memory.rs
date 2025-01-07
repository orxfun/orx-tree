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
/// As opposed to `Auto`, lazy approach has the following pros & cons:
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

    // # 1 - BUILD UP

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11

    let mut tree = DynTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();
    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree).push(8);

    let mut n3 = id3.node_mut(&mut tree);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree).push(9);
    id7.node_mut(&mut tree).extend([10, 11]);

    // since the tree only grew, we know that all id's above are valid
    // assert!(id2.is_valid_for(&tree));
}
