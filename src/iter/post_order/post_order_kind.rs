use super::depth_nodes::DepthNodes;
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait PostOrderKind<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    /// Element type of the iterator; i.e., `Iterator::Item`.
    type YieldElement;

    /// Mutable element type of the iterator; i.e., `Iterator::Item`.
    type YieldElementMut;

    /// Creates the element to be yield, or the iterator item, from the queue element.
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        ptr: NodePtr<V>,
        depth: usize,
        depth_nodes: &DepthNodes<V>,
    ) -> Self::YieldElement;

    /// Creates the mutable element to be yield, or the iterator item, from the queue element.
    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        ptr: NodePtr<V>,
        depth: usize,
        depth_nodes: &DepthNodes<V>,
    ) -> Self::YieldElementMut;
}
