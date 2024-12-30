use super::{DataFromNode, IterKindCore, NodeDepthSiblingVal, NodeDepthVal, NodeFromNode, NodeVal};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub trait IterOver {
    type IterKind<'a, V, M, P>: IterKindCore<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

// data

pub struct OverData;

impl IterOver for OverData {
    type IterKind<'a, V, M, P>
        = NodeVal<DataFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

pub struct OverDepthData;

impl IterOver for OverDepthData {
    type IterKind<'a, V, M, P>
        = NodeDepthVal<DataFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

pub struct OverDepthSiblingData;

impl IterOver for OverDepthSiblingData {
    type IterKind<'a, V, M, P>
        = NodeDepthSiblingVal<DataFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

// node

pub struct OverNode;

impl IterOver for OverNode {
    type IterKind<'a, V, M, P>
        = NodeVal<NodeFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

pub struct OverDepthNode;

impl IterOver for OverDepthNode {
    type IterKind<'a, V, M, P>
        = NodeDepthVal<NodeFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

pub struct OverDepthSiblingNode;

impl IterOver for OverDepthSiblingNode {
    type IterKind<'a, V, M, P>
        = NodeDepthSiblingVal<NodeFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}
