use super::{Enumerator, NodeData};
use crate::{helpers::N, iter::NodeDataData, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait Element {
    type Enumeration: Enumerator;

    type NodeData: NodeData;

    fn element<'a, V, M, P>(
        col: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> <Self::Enumeration as Enumerator>::Output<<Self::NodeData as NodeData>::Value<'a, V, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        let x = <Self::NodeData as NodeData>::value(col, node_ptr);
        todo!()
    }

    fn element_mut<'a, V, M, P>(
        col: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
    ) -> <Self::Enumeration as Enumerator>::Output<
        <Self::NodeData as NodeData>::ValueMut<'a, V, M, P>,
    >
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
    {
        todo!()
    }
}
