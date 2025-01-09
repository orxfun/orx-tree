use super::states::States;
use crate::{
    memory::MemoryPolicy,
    pinned_storage::PinnedStorage,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemMut, OverMut},
        traverser_core::TraverserCore,
        Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};
use core::marker::PhantomData;

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
pub struct PostOrder<O = OverData>
where
    O: Over,
{
    states: States,
    phantom: PhantomData<O>,
}

impl Default for PostOrder {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> Traverser<O> for PostOrder<O>
where
    O: Over,
{
    type IntoOver<O2>
        = PostOrder<O2>
    where
        O2: Over;

    fn new() -> Self {
        Self {
            states: Default::default(),
            phantom: PhantomData,
        }
    }

    fn iter<'a, V, M, P>(
        &'a mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        let states = self.states.for_variant::<V>();
        Self::iter_with_storage(node, states)
    }

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        PostOrder {
            states: self.states,
            phantom: PhantomData,
        }
    }

    fn iter_mut<'a, V, M, P>(
        &'a mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut,
    {
        let states = self.states.for_variant::<V>();
        Self::iter_mut_with_storage(node_mut, states)
    }

    fn into_iter<'a, V, M, P>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = crate::traversal::over_mut::OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut,
    {
        let states = self.states.for_variant::<V>();
        Self::into_iter_with_storage(node_mut, states)
    }
}
