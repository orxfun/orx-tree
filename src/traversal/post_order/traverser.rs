use super::{
    into_iter::PostOrderIterInto, iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr,
    iter_ref::PostOrderIterRef, states::States,
};
use crate::{
    memory::MemoryPolicy,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemMut, OverMut},
        Traverser,
    },
    Dyn, NodeMut, NodeRef, TreeVariant,
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
    states: States<Dyn<i32>>,
    phantom: PhantomData<O>,
}

impl<O> Default for PostOrder<O>
where
    O: Over,
{
    fn default() -> Self {
        Self {
            states: Default::default(),
            phantom: PhantomData,
        }
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

    fn iter<'a, V, M, P>(
        &'a mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: 'a,
        Self: 'a,
    {
        // let root = node.node_ptr().clone();
        // let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((&mut self.states, root));
        // PostOrderIterRef::from((node.col(), iter_ptr))
        core::iter::empty()
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
        O: OverMut + 'a,
        Self: 'a,
    {
        // let root = node_mut.node_ptr().clone();
        // let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((&mut self.states, root));
        // unsafe { PostOrderIterMut::from((node_mut.col(), iter_ptr)) }
        core::iter::empty()
    }

    fn into_iter<'a, V, M, P>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = crate::traversal::over_mut::OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut + 'a,
        Self: 'a,
    {
        // let (col, root) = node_mut.into_inner();

        // let iter_ptr =
        //     PostOrderIterPtr::<V, O::Enumeration, _>::from((&mut self.states, root.clone()));
        // unsafe { PostOrderIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
        core::iter::empty()
    }
}
