use super::{depth_nodes::DepthNodes, post_order_kind::PostOrderKind};
use crate::{helpers::N, iter::NodeValue, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Iterator over values obtained from tree nodes.
pub struct PostNodeVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> PostOrderKind<'a, V, M, P> for PostNodeVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: NodeValue<'a, V, M, P>,
{
    type YieldElement = D::Value;

    type YieldElementMut = D::ValueMut;

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        ptr: NodePtr<V>,
        _: usize,
        _: &DepthNodes<V>,
    ) -> Self::YieldElement {
        let node = unsafe { &*ptr.ptr() };
        D::value(col, node)
    }

    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        ptr: NodePtr<V>,
        _: usize,
        _: &DepthNodes<V>,
    ) -> Self::YieldElementMut {
        let node = unsafe { &mut *ptr.ptr_mut() };
        D::value_mut(col, node)
    }
}
