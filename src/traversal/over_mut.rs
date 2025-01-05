use super::{
    enumeration::Enumeration,
    node_item_mut::NodeItemMut,
    over::{Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
};
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub type OverItemMut<'a, V, O, M = DefaultMemory<V>, P = DefaultPinVec<V>> =
    <<O as Over<V>>::Enumeration as Enumeration>::Item<<O as OverMut<V>>::NodeItemMut<'a, M, P>>;

pub trait OverMut<V: TreeVariant>: Over<V> {
    type NodeItemMut<'a, M, P>: NodeItemMut<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// val

impl<V: TreeVariant> OverMut<V> for OverData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// depth & val

impl<V: TreeVariant> OverMut<V> for OverDepthData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// sibling & val

impl<V: TreeVariant> OverMut<V> for OverSiblingIdxData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// depth & sibling & val

impl<V: TreeVariant> OverMut<V> for OverDepthSiblingIdxData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}
