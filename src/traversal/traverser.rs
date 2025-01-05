use super::{
    enumeration::Enumeration,
    node_item::NodeItem,
    node_item_mut::NodeItemMut,
    over::{Over, OverItem},
};
use crate::{helpers::N, NodeMut, NodeRef, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub trait Traverser<V: TreeVariant> {
    type Over: Over<V>;

    fn iter<'a, M, P>(
        &mut self,
        node: &impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, Self::Over, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        Self: 'a;
}

// pub trait TraverserMut<V: TreeVariant>: Traverser<V> {
//     type NodeItemMut<'a, M, P>: NodeItemMut<'a, V, M, P>
//     where
//         M: MemoryPolicy<V> + 'a,
//         P: PinnedVec<N<V>> + 'a,
//         V: 'a,
//         Self: 'a;

//     fn iter_mut<'a, M, P>(
//         &mut self,
//         node_mut: &mut NodeMut<'a, V, M, P>,
//     ) -> impl Iterator<Item = <Self::ItemKind as Enumeration>::Item<Self::NodeItemMut<'a, M, P>>>
//     where
//         V: TreeVariant + 'a,
//         M: MemoryPolicy<V> + 'a,
//         P: PinnedVec<N<V>> + 'a,
//         Self: 'a;
// }
