use super::{
    enumeration::Enumeration,
    node_item_mut::NodeItemMut,
    over::{Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
};
use crate::{
    TreeVariant,
    memory::{Auto, MemoryPolicy},
    pinned_storage::{PinnedStorage, SplitRecursive},
};
use orx_selfref_col::Variant;

pub type OverItemMut<'a, V, O, M = Auto, P = SplitRecursive> =
    <<O as Over>::Enumeration as Enumeration>::Item<<O as OverMut>::NodeItemMut<'a, V, M, P>>;

pub type OverItemInto<'a, V, O> =
    <<O as Over>::Enumeration as Enumeration>::Item<<V as Variant>::Item>;

/// Type that defines the type of the mutable items that iterators created by a traverser such as the [`Dfs`] or [`PostOrder`].
///
/// [`Dfs`]: crate::traversal::Dfs
/// [`PostOrder`]: crate::traversal::PostOrder
pub trait OverMut: Over {
    /// Part of the mutable iterator item which only depends on the node's internal data.
    type NodeItemMut<'a, V, M, P>: NodeItemMut<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;
}

// val

impl OverMut for OverData {
    type NodeItemMut<'a, V, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;
}

// depth & val

impl OverMut for OverDepthData {
    type NodeItemMut<'a, V, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;
}

// sibling & val

impl OverMut for OverSiblingIdxData {
    type NodeItemMut<'a, V, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;
}

// depth & sibling & val

impl OverMut for OverDepthSiblingIdxData {
    type NodeItemMut<'a, V, M, P>
        = &'a mut V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;
}
