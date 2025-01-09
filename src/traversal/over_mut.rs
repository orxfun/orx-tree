use orx_selfref_col::Variant;

use super::{
    enumeration::Enumeration,
    node_item_mut::NodeItemMut,
    over::{Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
};
use crate::{
    memory::{Auto, MemoryPolicy},
    pinned_storage::{PinnedStorage, SplitRecursive},
    TreeVariant,
};

pub type OverItemMut<'a, V, O, M = Auto, P = SplitRecursive> =
    <<O as Over<V>>::Enumeration as Enumeration>::Item<<O as OverMut<V>>::NodeItemMut<'a, M, P>>;

pub type OverItemInto<'a, V, O> =
    <<O as Over<V>>::Enumeration as Enumeration>::Item<<V as Variant>::Item>;

/// Type that defines the type of the mutable items that iterators created by a traverser such as the [`Dfs`] or [`PostOrder`].
///
/// [`Dfs`]: crate::traversal::Dfs
/// [`PostOrder`]: crate::traversal::PostOrder
pub trait OverMut<V: TreeVariant>: Over<V> {
    /// Part of the mutable iterator item which only depends on the node's internal data.
    type NodeItemMut<'a, M, P>: NodeItemMut<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: 'a,
        Self: 'a;
}

// val

impl<V: TreeVariant> OverMut<V> for OverData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: 'a,
        Self: 'a;
}

// depth & val

impl<V: TreeVariant> OverMut<V> for OverDepthData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: 'a,
        Self: 'a;
}

// sibling & val

impl<V: TreeVariant> OverMut<V> for OverSiblingIdxData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: 'a,
        Self: 'a;
}

// depth & sibling & val

impl<V: TreeVariant> OverMut<V> for OverDepthSiblingIdxData {
    type NodeItemMut<'a, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: 'a,
        Self: 'a;
}
