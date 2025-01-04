use super::DfsIterPtr;
use crate::helpers::N;
use crate::traversal::node_item::NodeItem;
use crate::{
    traversal::{element::Element, element_mut::ElementMut},
    TreeVariant,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

type Item<V, E> = <E as Element>::Item<NodePtr<V>>;
type Stack<V, E> = Vec<Item<V, E>>;

pub struct DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: Element,
    S: SoM<Stack<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    col: &'a mut SelfRefCol<V, M, P>,
    iter: DfsIterPtr<V, E, S>,
    phantom: PhantomData<D>,
}

// impl<'a, V, M, P, E, S, D> From<(&'a mut SelfRefCol<V, M, P>, DfsIterPtr<V, E, S>)>
//     for DfsIterMut<'a, V, M, P, E, S, D>
// where
//     V: TreeVariant,
//     M: MemoryPolicy<V>,
//     P: PinnedVec<N<V>>,
//     E: Element ,
//     S: SoM<Stack<V, E>>,
//     D: NodeItem<'a, V, M, P>,
// {
//     fn from((col, iter): (&'a mut SelfRefCol<V, M, P>, DfsIterPtr<V, E, S>)) -> Self {
//         Self {
//             col,
//             iter,
//             phantom: PhantomData,
//         }
//     }
// }

impl<'a, V, M, P, E, S, D> Iterator for DfsIterMut<'a, V, M, P, E, S, D>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    E: Element,
    S: SoM<Stack<V, E>>,
    D: NodeItem<'a, V, M, P>,
{
    type Item = E::Item<D>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(move |element: Item<V, E>| E::from_element_ptr(self.col, element))
    }
}
