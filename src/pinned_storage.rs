use crate::{helpers::N, TreeVariant};
use orx_split_vec::{PinnedVec, Recursive, SplitVec};

pub trait PinnedStorage: 'static {
    type PinnedVec<V>: PinnedVec<N<V>>
    where
        V: TreeVariant;
}

pub struct SplitRecursive;
impl PinnedStorage for SplitRecursive {
    type PinnedVec<V>
        = SplitVec<N<V>, Recursive>
    where
        V: TreeVariant;
}
