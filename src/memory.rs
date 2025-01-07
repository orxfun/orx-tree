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

// impl<V, M, P> Tree<V, M, P>
// where
//     V: TreeVariant,
//     M: MemoryPolicy<V>,
//     P: PinnedVec<N<V>>,
// {
// }

// impl<const D: usize, R, V, P> Tree<V, MemoryReclaimOnThreshold<D, V, R>, P>
// where
//     R: MemoryReclaimer<V>,
//     V: TreeVariant,
//     P: PinnedVec<N<V>>,
// {
//     pub fn into_lazy_reclaim(self) -> Tree<V, MemoryReclaimNever, P> {
//         Tree(self.0.into())
//     }
// }

// impl<V, P> Tree<V, MemoryReclaimNever, P>
// where
//     V: TreeVariant,
//     P: PinnedVec<N<V>>,
// {
//     pub fn into_auto_reclaim(self) -> Tree<V, DefaultMemory<V>, P> {
//         Tree(self.0.into())
//     }

//     pub fn into_auto_reclaim_with_threshold<const D: usize>(
//         self,
//     ) -> Tree<V, MemoryReclaimOnThreshold<D, V, V::Reclaimer>, P> {
//         Tree(self.0.into())
//     }
// }
