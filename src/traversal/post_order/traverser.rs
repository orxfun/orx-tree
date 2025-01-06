use super::{
    iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
    post_enumeration::PostOrderEnumeration, states::States,
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
    O::Enumeration: PostOrderEnumeration,
{
    states: States<V>,
    phantom: PhantomData<O>,
}

impl<V, O> Default for PostOrder<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: PostOrderEnumeration,
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
    O::Enumeration: PostOrderEnumeration,
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
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((&mut self.states, root));
        PostOrderIterRef::from((node.col(), iter_ptr))
    }
}

impl<V, O> TraverserMut<V, O> for PostOrder<V, O>
where
    V: TreeVariant,
    O: OverMut<V>,
    O::Enumeration: PostOrderEnumeration,
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

// transform

impl<V, O> PostOrder<V, O>
where
    V: TreeVariant,
    O: Over<V>,
    O::Enumeration: PostOrderEnumeration,
{
    fn transform<P>(self) -> PostOrder<V, P>
    where
        P: Over<V>,
        P::Enumeration: PostOrderEnumeration,
    {
        PostOrder {
            states: self.states,
            phantom: PhantomData,
        }
    }
}

// transform: over_nodes

impl<V> PostOrder<V, OverData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> PostOrder<V, OverNode> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverDepthData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> PostOrder<V, OverDepthNode> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverSiblingIdxData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> PostOrder<V, OverSiblingIdxNode> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverDepthSiblingIdxData>
where
    V: TreeVariant,
{
    pub fn over_nodes(self) -> PostOrder<V, OverDepthSiblingIdxNode> {
        self.transform()
    }
}

// transform: over_data

impl<V> PostOrder<V, OverNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> PostOrder<V, OverData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverDepthNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> PostOrder<V, OverDepthData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverSiblingIdxNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> PostOrder<V, OverSiblingIdxData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverDepthSiblingIdxNode>
where
    V: TreeVariant,
{
    pub fn over_data(self) -> PostOrder<V, OverDepthSiblingIdxData> {
        self.transform()
    }
}

// transform: with_depth

impl<V> PostOrder<V, OverData>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> PostOrder<V, OverDepthData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverNode>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> PostOrder<V, OverDepthNode> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverSiblingIdxData>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> PostOrder<V, OverDepthSiblingIdxData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverSiblingIdxNode>
where
    V: TreeVariant,
{
    pub fn with_depth(self) -> PostOrder<V, OverDepthSiblingIdxNode> {
        self.transform()
    }
}

// transform: with_sibling_idx

impl<V> PostOrder<V, OverData>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> PostOrder<V, OverSiblingIdxData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverNode>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> PostOrder<V, OverSiblingIdxNode> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverDepthData>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> PostOrder<V, OverDepthSiblingIdxData> {
        self.transform()
    }
}

impl<V> PostOrder<V, OverDepthNode>
where
    V: TreeVariant,
{
    pub fn with_sibling_idx(self) -> PostOrder<V, OverDepthSiblingIdxNode> {
        self.transform()
    }
}
