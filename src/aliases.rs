use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage};
use orx_selfref_col::SelfRefCol;

pub(crate) type N<V> = orx_selfref_col::Node<V>;

pub(crate) type Col<V, M, P> =
    SelfRefCol<V, <M as MemoryPolicy>::MemoryReclaimPolicy<V>, <P as PinnedStorage>::PinnedVec<V>>;
