use super::{into_iter::BfsIterInto, iter_mut::BfsIterMut, iter_ptr::BfsIterPtr, queue::Queue};
use crate::{
    memory::MemoryPolicy,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemInto, OverItemMut, OverMut},
        traverser::Traverser,
        traverser_core::TraverserCore,
    },
    NodeMut, NodeRef, TreeVariant,
};

/// A breadth first search traverser, also known as level-order
/// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
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
pub struct Bfs<O = OverData>
where
    O: Over,
{
    queue: Queue<O::Enumeration>,
}

impl Default for Bfs {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> Traverser<O> for Bfs<O>
where
    O: Over,
{
    type IntoOver<O2>
        = Bfs<O2>
    where
        O2: Over;

    fn new() -> Self {
        Self {
            queue: Default::default(),
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
        let queue = self.queue.for_variant::<V>();
        Self::iter_with_storage(node, queue)
    }

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        Bfs::<O2>::new()
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
        let queue = self.queue.for_variant::<V>();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((queue, root));
        unsafe { BfsIterMut::from((node_mut.col(), iter_ptr)) }
    }

    fn into_iter<'a, V, M, P>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut,
    {
        let (col, root) = node_mut.into_inner();
        let queue = self.queue.for_variant::<V>();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((queue, root.clone()));
        unsafe { BfsIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}
