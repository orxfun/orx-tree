use orx_selfref_col::SelfRefCol;

use crate::{memory::MemoryPolicy, pinned_storage::PinnedStorage};

pub type N<V> = orx_selfref_col::Node<V>;

pub(crate) type Col<V, M, P> = SelfRefCol<
    V,
    <M as MemoryPolicy>::MemoryReclaimPolicy<V>,
    <P as PinnedStorage>::PinnedVec<V>,
>;
