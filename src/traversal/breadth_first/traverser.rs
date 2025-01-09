use super::{
    into_iter::BfsIterInto, iter_mut::BfsIterMut, iter_ptr::BfsIterPtr, iter_ref::BfsIterRef,
    queue::Queue,
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
pub struct Bfs<V, O = OverData>
where
    V: TreeVariant,
    O: Over<V>,
{
    stack: Queue<V, O::Enumeration>,
}

impl<V, O> Default for Bfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}

impl<V, O> Traverser<V, O> for Bfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
{
    type IntoOver<O2>
        = Bfs<V, O2>
    where
        O2: Over<V>;

    fn iter<'a, M, P>(
        &mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: 'a,
        Self: 'a,
    {
        let root = node.node_ptr().clone();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        BfsIterRef::from((node.col(), iter_ptr))
    }

    fn transform_into<O2: Over<V>>(self) -> Self::IntoOver<O2> {
        Bfs::<V, O2>::default()
    }

    fn iter_mut<'a, M, P>(
        &mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut<V> + 'a,
        Self: 'a,
    {
        let root = node_mut.node_ptr().clone();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        unsafe { BfsIterMut::from((node_mut.col(), iter_ptr)) }
    }

    fn into_iter<'a, M, P>(
        &mut self,
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = crate::traversal::over_mut::OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut<V> + 'a,
        Self: 'a,
    {
        let (col, root) = node_mut.into_inner();

        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root.clone()));
        unsafe { BfsIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}
