use super::{iter_mut::DfsIterMut, iter_ptr::DfsIterPtr, iter_ref::DfsIterRef, Stack};
use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemMut, OverMut},
        traverser::Traverser,
        traverser_mut::TraverserMut,
    },
    NodeMut, NodeRef, TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// A depth first traverser ([Wikipedia](https://en.wikipedia.org/wiki/Depth-first_search)).
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
pub struct Dfs<V, O = OverData>
where
    V: TreeVariant,
    O: Over<V>,
{
    stack: Stack<V, O::Enumeration>,
}

impl<V, O> Default for Dfs<V, O>
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

impl<V, O> Traverser<V, O> for Dfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
{
    type IntoOver<O2>
        = Dfs<V, O2>
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
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        DfsIterRef::from((node.col(), iter_ptr))
    }

    fn over_data(self) -> Self::IntoOver<O::IntoOverData> {
        Dfs::<V, O::IntoOverData>::default()
    }

    fn over_nodes(self) -> Self::IntoOver<O::IntoOverNode> {
        Dfs::<V, O::IntoOverNode>::default()
    }

    fn with_depth(self) -> Self::IntoOver<O::IntoWithDepth> {
        Dfs::<V, O::IntoWithDepth>::default()
    }

    fn with_sibling_idx(self) -> Self::IntoOver<O::IntoWithSiblingIdx> {
        Dfs::<V, O::IntoWithSiblingIdx>::default()
    }
}

impl<V, O> TraverserMut<V, O> for Dfs<V, O>
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
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        unsafe { DfsIterMut::from((node_mut.col(), iter_ptr)) }
    }
}
