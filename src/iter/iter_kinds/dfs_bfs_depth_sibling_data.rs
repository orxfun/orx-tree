use super::{
    dfs_bfs_kind::{node, node_mut},
    DfsBfsIterKind, NodeValue, QueueElement,
};
use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

// core

/// Iterator over tuples of node depths, sibling indices and values obtained from tree nodes.
pub struct DfsBfsNodeDepthSiblingVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> DfsBfsIterKind<'a, V, M, P> for DfsBfsNodeDepthSiblingVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: NodeValue<'a, V, M, P>,
{
    type QueueElement = (usize, usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (
        usize,
        usize,
        <Self::ValueFromNode as NodeValue<'a, V, M, P>>::Value,
    );

    type YieldElementMut = (
        usize,
        usize,
        <Self::ValueFromNode as NodeValue<'a, V, M, P>>::ValueMut,
    );

    #[inline(always)]
    fn children(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .enumerate()
            .map(move |(i, ptr)| (depth, i, ptr.clone()))
    }

    #[inline(always)]
    fn children_rev(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .enumerate()
            .rev()
            .map(move |(i, ptr)| (depth, i, ptr.clone()))
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElement {
        (
            queue_element.0,
            queue_element.1,
            D::value(col, node(&queue_element.2)),
        )
    }

    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElementMut {
        (
            queue_element.0,
            queue_element.1,
            D::value_mut(col, node_mut(&queue_element.2)),
        )
    }
}