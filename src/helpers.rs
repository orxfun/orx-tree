use orx_selfref_col::SelfRefCol;

use crate::memory::TreeMemoryPolicy;

pub type N<V> = orx_selfref_col::Node<V>;

pub(crate) type Col<V, M, P> = SelfRefCol<V, <M as TreeMemoryPolicy>::MemoryReclaimPolicy<V>, P>;
