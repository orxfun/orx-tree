use super::{
    over_mut::{OverItemMut, OverMut},
    Traverser,
};
use crate::{helpers::N, NodeMut, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub trait TraverserMut<V: TreeVariant>: Traverser<V>
where
    Self::Over: OverMut<V>,
{
    fn iter_mut<'a, M, P>(
        &mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, Self::Over, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        Self: 'a;
}
