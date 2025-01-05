use super::{
    dfs_enumeration::DepthFirstEnumeration, iter_mut::DfsIterMut, iter_ref::DfsIterRef, DfsIterPtr,
    Stack,
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

pub struct Dfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: DepthFirstEnumeration,
{
    stack: Stack<V, O::Enumeration>,
}

impl<V, O> Default for Dfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: DepthFirstEnumeration,
{
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}

impl<V, O> Traverser<V> for Dfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: DepthFirstEnumeration,
{
    type Over = O;

    fn iter<'a, M, P>(
        &mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, Self::Over, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        Self: 'a,
    {
        let root = node.node_ptr().clone();
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        DfsIterRef::from((node.col(), iter_ptr))
    }
}

impl<V, O> TraverserMut<V> for Dfs<V, O>
where
    V: TreeVariant,
    O: OverMut<V>,
    O::Enumeration: DepthFirstEnumeration,
{
    fn iter_mut<'a, M, P>(
        &mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, Self::Over, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        Self: 'a,
    {
        let root = node_mut.node_ptr().clone();
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        unsafe { DfsIterMut::from((node_mut.col(), iter_ptr)) }
    }
}
