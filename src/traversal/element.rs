use super::{node_item::NodeItem, node_item_mut::NodeItemMut};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait Element {
    type Item<D>;

    fn from_root<D>(root: D) -> Self::Item<D>;

    fn node_value<D>(element: &Self::Item<D>) -> &D;

    fn children<D>(
        parent: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>>;

    fn map<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
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
        Self::map(element_ptr, map)
    }

    fn from_element_ptr_mut<'a, V, M, P, E>(
        col: &'a mut SelfRefCol<V, M, P>,
        element_ptr: Self::Item<NodePtr<V>>,
    ) -> Self::Item<E>
    where
        V: TreeVariant,
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
        E: NodeItemMut<'a, V, M, P>,
    {
        let map = |ptr| E::from_ptr(col, ptr);
        Self::map(element_ptr, map)
    }
}
