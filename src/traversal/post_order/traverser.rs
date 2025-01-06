use super::{
    iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
    states::States,
};
use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemMut, OverMut},
        traverser_mut::TraverserMut,
        Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// A post order traverser ([Wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
///
/// A traverser can be created once and used to traverse over trees multiple times without
/// requiring additional memory allocation.
///
/// # Construction
///
/// A post order traverser can be created,
/// * either by using Default trait and providing its two generic type parameters
///   * `PostOrder::<_, OverData>::default()` or `PostOrder::<_, OverDepthSiblingIdxData>::default()`, or
///   * `PostOrder::<Dyn<u64>, OverData>::default()` or `PostOrder::<Dary<2, String>, OverDepthSiblingIdxData>::default()`
///     if we want the complete type signature.
/// * or by using the [`Traversal`] type.
///   * `Traversal.post_order()` or `Traversal.post_order().with_depth().with_sibling_idx()`.
///
/// [`Traversal`]: crate::Traversal
pub struct PostOrder<V, O = OverData>
where
    V: TreeVariant,
    O: Over<V>,
{
    states: States<V>,
    phantom: PhantomData<O>,
}

impl<V, O> Default for PostOrder<V, O>
where
    V: TreeVariant,
    O: Over<V>,
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
{
    type IntoOver<O2>
        = PostOrder<V, O2>
    where
        O2: Over<V>;

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

    fn transform_into<O2: Over<V>>(self) -> Self::IntoOver<O2> {
        PostOrder::<V, O2>::default()
    }

    fn over_data(self) -> Self::IntoOver<O::IntoOverData> {
        PostOrder::<V, O::IntoOverData>::default()
    }

    fn over_nodes(self) -> Self::IntoOver<O::IntoOverNode> {
        PostOrder::<V, O::IntoOverNode>::default()
    }

    fn with_depth(self) -> Self::IntoOver<O::IntoWithDepth> {
        PostOrder::<V, O::IntoWithDepth>::default()
    }

    fn with_sibling_idx(self) -> Self::IntoOver<O::IntoWithSiblingIdx> {
        PostOrder::<V, O::IntoWithSiblingIdx>::default()
    }
}

impl<V, O> TraverserMut<V, O> for PostOrder<V, O>
where
    V: TreeVariant,
    O: OverMut<V>,
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
