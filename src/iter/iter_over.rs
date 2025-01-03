use super::{post_order::PostOrderKind, DfsBfsIterKind, QueueElement};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// Defines the return element or item of the stack/queue based iterators over the tree,
/// such as the depth first or breadth first traversals.
///
/// * [`OverData`] yields data of nodes, which might be [`data`] or [`data_mut`] depending on whether or not the iterator is mutable
/// * [`OverDepthData`] yields (depth, data) pairs where the first element is a usize representing the depth of the node in the tree
/// * [`OverDepthSiblingData`] yields (depth, sibling_idx, data) tuples where the second element is a usize representing the index of the node among its siblings
/// * [`OverNode`] yields directly the nodes ([`Node`])
/// * [`OverDepthNode`] yields (depth, node) pairs where the first element is a usize representing the depth of the node in the tree
/// * [`OverDepthSiblingNode`] yields (depth, sibling_idx, node) tuples where the second element is a usize representing the index of the node among its siblings
///
/// Finally, if we require to iterate over pointers to the nodes, we can use [`OverPtr`], [`OverDepthPtr`] and [`OverDepthSiblingPtr`].
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
/// [`Node`]: crate::Node
/// [`OverData`]: crate::iter::OverData
/// [`OverDepthData`]: crate::iter::OverDepthData
/// [`OverDepthSiblingData`]: crate::iter::OverDepthSiblingData
/// [`OverNode`]: crate::iter::OverNode
/// [`OverDepthNode`]: crate::iter::OverDepthNode
/// [`OverDepthSiblingNode`]: crate::iter::OverDepthSiblingNode
/// [`OverPtr`]: crate::iter::OverPtr
/// [`OverDepthPtr`]: crate::iter::OverDepthPtr
/// [`OverDepthSiblingPtr`]: crate::iter::OverDepthSiblingPtr
pub trait IterOver {
    /// Core iteration kind for stack/queue based iterators such as dfs & bfs.
    type DfsBfsIterKind<'a, V, M, P>: DfsBfsIterKind<
        'a,
        V,
        M,
        P,
        QueueElement = Self::DfsBfsQueueElement<V>,
    >
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    /// Core iteration kind for depth-sized-vector-based iterators such as post-order.
    type PostOrderKind<'a, V, M, P>: PostOrderKind<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    /// Type of elements stored in the intermediate queue storage.
    type DfsBfsQueueElement<V>: QueueElement<V>
    where
        V: TreeVariant;
}

/// Defines the return element or item of the mutable iterator over the tree.
pub trait IterMutOver: IterOver {}
