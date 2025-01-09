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
    pub(super) states: States,
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

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        PostOrder {
            states: self.states,
            phantom: PhantomData,
        }
    }
}
