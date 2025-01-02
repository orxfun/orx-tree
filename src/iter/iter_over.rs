use super::{post_order::PostOrderKind, DfsBfsIterKind, QueueElement};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// Defines the return element or item of the stack/queue based iterators over the tree,
/// such as the depth first or breadth first traversals.
pub trait IterOver {
    /// Core iteration kind.
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
