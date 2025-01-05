use super::over::{Over, OverItem};
use crate::{helpers::N, NodeRef, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub trait Traverser<V: TreeVariant> {
    type Over: Over<V>;

    fn iter<'a, M, P>(
        &mut self,
        node: &impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, Self::Over, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        Self: 'a;
}
