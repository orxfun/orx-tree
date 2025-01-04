use super::element::Element;
use crate::{helpers::N, Node, NodeMut, NodeRef, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait Traverser {
    type ItemKind: Element;

    type NodeData;

    type NodeDataMut;

    fn iter<'a, V, M, P>(
        &mut self,
        node: &impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = <Self::ItemKind as Element>::Item<Self::NodeData>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    fn iter_mut<'a, V, M, P>(
        &mut self,
        node_mut: &mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = <Self::ItemKind as Element>::Item<Self::NodeDataMut>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}
