use crate::TreeVariant;
use orx_selfref_col::{MemoryPolicy, MemoryReclaimNever, MemoryReclaimOnThreshold};

pub trait TreeMemoryPolicy: 'static {
    type MemoryReclaimPolicy<V>: MemoryPolicy<V>
    where
        V: TreeVariant;
}

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
