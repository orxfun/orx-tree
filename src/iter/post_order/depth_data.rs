use super::{depth_nodes::DepthNodes, post_order_kind::PostOrderKind};
use crate::{helpers::N, iter::NodeValue, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Iterator over values obtained from tree nodes.
pub struct NodeDepthVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> PostOrderKind<'a, V, M, P> for NodeDepthVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: NodeValue<'a, V, M, P>,
{
    type YieldElement = (usize, D::Value);

    type YieldElementMut = (usize, D::ValueMut);

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        ptr: NodePtr<V>,
        depth: usize,
        _: &DepthNodes<V>,
    ) -> Self::YieldElement {
        let node = unsafe { &*ptr.ptr() };
        (depth, D::value(col, node))
    }

    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        ptr: NodePtr<V>,
        depth: usize,
        _: &DepthNodes<V>,
    ) -> Self::YieldElementMut {
        let node = unsafe { &mut *ptr.ptr_mut() };
        (depth, D::value_mut(col, node))
    }
}
