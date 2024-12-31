use super::{
    kind_traits::node, DataFromNode, IterKindCore, IterOver, NodeFromNode, StackElement,
    ValueFromNode,
};
use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

// core

pub struct NodeDepthSiblingVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterKindCore<'a, V, M, P> for NodeDepthSiblingVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V, M, P>,
{
    type StackElement = (usize, usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (
        usize,
        usize,
        <Self::ValueFromNode as ValueFromNode<'a, V, M, P>>::Value,
    );

    #[inline(always)]
    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
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
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement {
        (
            stack_element.0,
            stack_element.1,
            D::value(col, node(&stack_element.2)),
        )
    }
}

// over

pub struct OverDepthSiblingData;

impl IterOver for OverDepthSiblingData {
    type IterKind<'a, V, M, P>
        = NodeDepthSiblingVal<DataFromNode>
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
