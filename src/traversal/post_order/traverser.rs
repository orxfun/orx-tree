use super::{
    iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
    post_enumeration::PostOrderEnumeration, states::States,
};
use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    traversal::{
        over::{Over, OverItem},
        over_mut::{OverItemMut, OverMut},
        traverser_mut::TraverserMut,
        Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub struct PostOrder<V>
where
    V: TreeVariant,
{
    states: States<V>,
}

impl<V> Default for PostOrder<V>
where
    V: TreeVariant,
{
    fn default() -> Self {
        Self {
            states: Default::default(),
        }
    }
}

impl<V, O> Traverser<V, O> for PostOrder<V>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: PostOrderEnumeration,
{
    fn iter<'a, M, P>(
        &mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        O: 'a,
        Self: 'a,
    {
        let root = node.node_ptr().clone();
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((&mut self.states, root));
        PostOrderIterRef::from((node.col(), iter_ptr))
    }
}
