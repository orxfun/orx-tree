use super::{node_item::NodeItem, DepthSiblingIdxVal, DepthVal, Enumeration, SiblingIdxVal, Val};
use crate::{helpers::N, Node, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub trait Over<V: TreeVariant> {
    type Enumeration: Enumeration;

    type NodeItem<'a, M, P>: NodeItem<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// val

pub struct OverData;

impl<V: TreeVariant> Over<V> for OverData {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

pub struct OverNode;

impl<V: TreeVariant> Over<V> for OverNode {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// depth & val

pub struct OverDepthData;

impl<V: TreeVariant> Over<V> for OverDepthData {
    type Enumeration = DepthVal;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

pub struct OverDepthNode;

impl<V: TreeVariant> Over<V> for OverDepthNode {
    type Enumeration = DepthVal;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// sibling & val

pub struct OverSiblingIdxData;

impl<V: TreeVariant> Over<V> for OverSiblingIdxData {
    type Enumeration = SiblingIdxVal;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

pub struct OverSiblingIdxNode;

impl<V: TreeVariant> Over<V> for OverSiblingIdxNode {
    type Enumeration = SiblingIdxVal;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

// depth & sibling & val

pub struct OverDepthSiblingIdxData;

impl<V: TreeVariant> Over<V> for OverDepthSiblingIdxData {
    type Enumeration = DepthSiblingIdxVal;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}

pub struct OverDepthSiblingIdxNode;

impl<V: TreeVariant> Over<V> for OverDepthSiblingIdxNode {
    type Enumeration = DepthSiblingIdxVal;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;
}
