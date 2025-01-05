use super::{
    dfs_enumeration::DepthFirstEnumeration, iter_mut::DfsIterMut, iter_ptr::DfsIterPtr,
    iter_ref::DfsIterRef, Stack,
};
use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    traversal::{
        over::{
            Over, OverData, OverDepthData, OverDepthNode, OverDepthSiblingIdxData,
            OverDepthSiblingIdxNode, OverItem, OverNode, OverSiblingIdxData, OverSiblingIdxNode,
        },
        over_mut::{OverItemMut, OverMut},
        traverser::Traverser,
        traverser_mut::TraverserMut,
    },
    NodeMut, NodeRef, TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub struct Dfs<V, O = OverData>
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

impl<V, O> Traverser<V, O> for Dfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: DepthFirstEnumeration,
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
        let iter_ptr = DfsIterPtr::<V, O::Enumeration, _>::from((&mut self.stack, root));
        DfsIterRef::from((node.col(), iter_ptr))
    }
}

impl<V, O> TraverserMut<V, O> for Dfs<V, O>
where
    V: TreeVariant,
    O: OverMut<V>,
    O::Enumeration: DepthFirstEnumeration,
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

// transform

impl<V, O> Dfs<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: DepthFirstEnumeration,
{
    fn transform<P>(self) -> Dfs<V, P>
    where
        P: Over<V, Enumeration = O::Enumeration>,
    {
        Dfs { stack: self.stack }
    }
}

// transform: over_nodes

impl<V> Dfs<V, OverData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> Dfs<V, OverNode> {
        self.transform()
    }
}

impl<V> Dfs<V, OverDepthData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> Dfs<V, OverDepthNode> {
        self.transform()
    }
}

impl<V> Dfs<V, OverSiblingIdxData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> Dfs<V, OverSiblingIdxNode> {
        self.transform()
    }
}

impl<V> Dfs<V, OverDepthSiblingIdxData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> Dfs<V, OverDepthSiblingIdxNode> {
        self.transform()
    }
}

// transform: over_data

impl<V> Dfs<V, OverNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> Dfs<V, OverData> {
        self.transform()
    }
}

impl<V> Dfs<V, OverDepthNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> Dfs<V, OverDepthData> {
        self.transform()
    }
}

impl<V> Dfs<V, OverSiblingIdxNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> Dfs<V, OverSiblingIdxData> {
        self.transform()
    }
}

impl<V> Dfs<V, OverDepthSiblingIdxNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> Dfs<V, OverDepthSiblingIdxData> {
        self.transform()
    }
}

// transform: with_depth

impl<V> Dfs<V, OverData>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> Dfs<V, OverDepthData> {
        Default::default()
    }
}

impl<V> Dfs<V, OverNode>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> Dfs<V, OverDepthNode> {
        Default::default()
    }
}

impl<V> Dfs<V, OverSiblingIdxData>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> Dfs<V, OverDepthSiblingIdxData> {
        Default::default()
    }
}

impl<V> Dfs<V, OverSiblingIdxNode>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> Dfs<V, OverDepthSiblingIdxNode> {
        Default::default()
    }
}

// transform: with_sibling_idx

impl<V> Dfs<V, OverData>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> Dfs<V, OverSiblingIdxData> {
        Default::default()
    }
}

impl<V> Dfs<V, OverNode>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> Dfs<V, OverSiblingIdxNode> {
        Default::default()
    }
}

impl<V> Dfs<V, OverDepthData>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> Dfs<V, OverDepthSiblingIdxData> {
        Default::default()
    }
}

impl<V> Dfs<V, OverDepthNode>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> Dfs<V, OverDepthSiblingIdxNode> {
        Default::default()
    }
}
