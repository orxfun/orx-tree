use crate::{TreeVariant, aliases::N};
use orx_split_vec::{PinnedVec, Recursive, SplitVec};

/// Trait defining the underlying pinned vector storage of the tree.
pub trait PinnedStorage: 'static {
    /// The pinned vector for the given variant `V`.
    type PinnedVec<V>: PinnedVec<N<V>>
    where
        V: TreeVariant;
}

/// The tree uses `SplitVec<N<V>, Recursive>` as the underlying storage.
pub struct SplitRecursive;
impl PinnedStorage for SplitRecursive {
    type PinnedVec<V>
        = SplitVec<N<V>, Recursive>
    where
        V: TreeVariant;
}
