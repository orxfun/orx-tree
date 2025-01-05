use super::{node_item::NodeItem, node_item_mut::NodeItemMut};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait Element {
    type Item<D>;

    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E;

    fn from_element_ptr<'a, V, M, P, E>(
        col: &'a SelfRefCol<V, M, P>,
        element_ptr: Self::Item<NodePtr<V>>,
    ) -> Self::Item<E>
    where
        V: TreeVariant,
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
        E: NodeItem<'a, V, M, P>,
    {
        let map = |ptr| E::from_ptr(col, ptr);
        Self::map_node_data(element_ptr, map)
    }

    fn from_element_ptr_mut<'a, V, M, P, E>(
        col: &'a SelfRefCol<V, M, P>,
        element_ptr: Self::Item<NodePtr<V>>,
    ) -> Self::Item<E>
    where
        V: TreeVariant,
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
        E: NodeItemMut<'a, V, M, P>,
    {
        let map = |ptr: NodePtr<V>| E::from_ptr(col, ptr);
        Self::map_node_data(element_ptr, map)
    }
}
