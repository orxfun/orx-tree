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
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub struct PostOrder<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: PostOrderEnumeration,
{
    states: States<V>,
    phantom: PhantomData<O>,
}

impl<V, O> Default for PostOrder<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: PostOrderEnumeration,
{
    fn default() -> Self {
        Self {
            states: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<V, O> Traverser<V, O> for PostOrder<V, O>
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

impl<V, O> TraverserMut<V, O> for PostOrder<V, O>
where
    V: TreeVariant,
    O: OverMut<V>,
    O::Enumeration: PostOrderEnumeration,
{
    fn iter_mut<'a, M, P>(
        &mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        O: 'a,
        Self: 'a,
    {
        let root = node_mut.node_ptr().clone();
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((&mut self.states, root));
        unsafe { PostOrderIterMut::from((node_mut.col(), iter_ptr)) }
    }
}
