use super::{
    into_iter::DfsIterInto, iter_mut::DfsIterMut, iter_ptr::DfsIterPtr, iter_ref::DfsIterRef,
    stack::Stack,
};
use crate::{
    memory::MemoryPolicy,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemMut, OverMut},
        traverser::Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};

/// A depth first search traverser ([Wikipedia](https://en.wikipedia.org/wiki/Depth-first_search)).
///
/// A traverser can be created once and used to traverse over trees multiple times without
/// requiring additional memory allocation.
///
/// # Construction
///
/// A depth first traverser can be created,
/// * either by using Default trait and providing its two generic type parameters
///   * `Dfs::<_, OverData>::default()` or `Dfs::<_, OverDepthSiblingIdxData>::default()`, or
///   * `Dfs::<Dyn<u64>, OverData>::default()` or `Dfs::<Dary<2, String>, OverDepthSiblingIdxData>::default()`
///     if we want the complete type signature.
/// * or by using the [`Traversal`] type.
///   * `Traversal.dfs()` or `Traversal.dfs().with_depth().with_sibling_idx()`.
///
/// [`Traversal`]: crate::Traversal
pub struct Dfs<O = OverData>
where
    O: Over,
{
    stack: Stack<O::Enumeration>,
}

impl Default for Dfs {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> Traverser<O> for Dfs<O>
where
    O: Over,
{
    type IntoOver<O2>
        = Dfs<O2>
    where
        O2: Over;

    fn new() -> Self {
        Self {
            stack: Default::default(),
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
        let root = node.node_ptr().clone();
        let stack = self.stack.for_variant::<V>();
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((stack, root));
        DfsIterRef::from((node.col(), iter_ptr))
    }

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        Dfs::<O2>::new()
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
        let root = node_mut.node_ptr().clone();
        let stack = self.stack.for_variant::<V>();
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((stack, root));
        unsafe { DfsIterMut::from((node_mut.col(), iter_ptr)) }
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
        let (col, root) = node_mut.into_inner();
        let stack = self.stack.for_variant::<V>();
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((stack, root.clone()));
        unsafe { DfsIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}
