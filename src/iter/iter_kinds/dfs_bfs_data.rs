use super::{
    dfs_bfs_kind::{node, node_mut},
    DfsBfsIterKind, NodeValue, QueueElement,
};
use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

// core

/// Iterator over values obtained from tree nodes.
pub struct DfsBfsNodeVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> DfsBfsIterKind<'a, V, M, P> for DfsBfsNodeVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: NodeValue<'a, V, M, P>,
{
    type QueueElement = NodePtr<V>;

    type ValueFromNode = D;

    type YieldElement = <Self::ValueFromNode as NodeValue<'a, V, M, P>>::Value;

    type YieldElementMut = <Self::ValueFromNode as NodeValue<'a, V, M, P>>::ValueMut;

    #[inline(always)]
    fn children(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a {
        node(parent.node_ptr()).next().children_ptr().cloned()
    }

    #[inline(always)]
    fn children_rev(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a {
        node(parent.node_ptr()).next().children_ptr().rev().cloned()
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElement {
        D::value(col, node(queue_element))
    }

    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElementMut {
        D::value_mut(col, node_mut(queue_element))
    }
}
