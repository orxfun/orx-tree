use super::{
    kind_traits::node, DataFromNode, IterKindCore, IterOver, NodeFromNode, StackElement,
    ValueFromNode,
};
use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

// core

pub struct NodeVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterKindCore<'a, V, M, P> for NodeVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V, M, P>,
{
    type StackElement = NodePtr<V>;

    type ValueFromNode = D;

    type YieldElement = <Self::ValueFromNode as ValueFromNode<'a, V, M, P>>::Value;

    #[inline(always)]
    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        node(parent.node_ptr()).next().children_ptr().rev().cloned()
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement {
        D::value(col, node(&stack_element))
    }
}

// over

pub struct OverData;

impl IterOver for OverData {
    type IterKind<'a, V, M, P>
        = NodeVal<DataFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

pub struct OverNode;

impl IterOver for OverNode {
    type IterKind<'a, V, M, P>
        = NodeVal<NodeFromNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}
