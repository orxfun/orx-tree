use super::{
    OverData, OverMut,
    enumeration::Enumeration,
    over::{Over, OverItem},
    over_mut::{OverItemInto, OverItemMut},
};
use crate::{
    NodeMut, NodeMutOrientation, NodeRef, TreeVariant, memory::MemoryPolicy,
    pinned_storage::PinnedStorage,
};
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

pub trait TraverserCore<O = OverData>: Sized
where
    O: Over,
{
    type Storage<V>: Default
    where
        V: TreeVariant;

    fn storage_mut<V: TreeVariant>(&mut self) -> &mut Self::Storage<V>;

    fn iter_ptr_with_storage<'t, V>(
        node_ptr: NodePtr<V>,
        storage: impl SoM<Self::Storage<V>> + 't,
    ) -> impl Iterator<Item = <O::Enumeration as Enumeration>::Item<NodePtr<V>>>
    where
        V: TreeVariant + 't;

    fn iter_with_storage<'t, 'a, V, M, P>(
        node: &impl NodeRef<'a, V, M, P>,
        storage: impl SoM<Self::Storage<V>> + 't,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        Self::Storage<V>: 't,
        'a: 't;

    /// Returns an iterator which yields all nodes including the `node` and all its descendants; i.e.,
    /// all nodes of the subtree rooted at the given `node`.
    ///
    /// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
    /// breadth-first are the most well-known traversals.
    ///
    /// Typically, the `iter` call of a traverser does not require any memory allocation.
    ///
    /// # Yields
    ///
    /// The return value of the iterations depend on the second generic parameter of the traverser which implements
    /// the [`Over`] trait. The following is the complete list of implementations and the corresponding item type
    /// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
    /// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
    /// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
    /// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
    ///
    /// | Over | Yields | Depth First Example |
    /// |---|---|---|
    /// | [`OverData`] | &data | `Traversal.dfs()` |
    /// | [`OverDepthData`] | (depth, &data) | `Traversal.dfs().with_depth()` |
    /// | [`OverSiblingIdxData`] | (sibling_idx, &data) | `Traversal.dfs().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, &data) | `Traversal.with_depth().with_sibling_idx()` |
    /// | [`OverNode`] | Node | `Traversal.dfs().over_nodes()` |
    /// | [`OverDepthNode`] | (depth, Node) | `Traversal.dfs().over_nodes().with_depth()` |
    /// | [`OverSiblingIdxNode`] | (sibling_idx, Node) | `Traversal.dfs().over_nodes().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxNode`] | (depth, sibling_idx, Node) | `Traversal.dfs().over_nodes().with_depth().with_sibling_idx()` |
    ///
    /// [`Traversal`]: crate::traversal::Traversal
    /// [`OverData`]: crate::traversal::OverData
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    /// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
    /// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
    /// [`OverNode`]: crate::traversal::OverNode
    /// [`OverDepthNode`]: crate::traversal::OverDepthNode
    /// [`OverSiblingIdxNode`]: crate::traversal::OverSiblingIdxNode
    /// [`OverDepthSiblingIdxNode`]: crate::traversal::OverDepthSiblingIdxNode
    fn iter<'t, 'a, V, M, P>(
        &'t mut self,
        node: &impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        'a: 't;

    fn iter_mut_with_storage<'t, 'a, V, M, P, MO>(
        node: &mut NodeMut<'a, V, M, P, MO>,
        storage: impl SoM<Self::Storage<V>> + 't,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
        Self::Storage<V>: 't,
        'a: 't;

    /// Returns a mutable iterator which yields all nodes including the `node` and all its descendants; i.e.,
    /// all nodes of the subtree rooted at the given `node`.
    ///
    /// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
    /// breadth-first are the most well-known traversals.
    ///
    /// Typically, the `iter` or `iter_mut` or `into_iter` call of a traverser does not require any memory allocation.
    ///
    /// # Yields
    ///
    /// The return value of the iterations depend on the second generic parameter of the traverser which implements
    /// the [`OverMut`] trait. The following is the complete list of implementations and the corresponding item type
    /// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
    /// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
    /// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
    /// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
    ///
    /// | Over | Yields | Depth First Example |
    /// |---|---|---|
    /// | [`OverData`] | &mut data | `Traversal.dfs()` |
    /// | [`OverDepthData`] | (depth, &mut data) | `Traversal.dfs().with_depth()` |
    /// | [`OverSiblingIdxData`] | (sibling_idx, &mut data) | `Traversal.dfs().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, &mut data) | `Traversal.with_depth().with_sibling_idx()` |
    ///
    /// [`Traversal`]: crate::traversal::Traversal
    /// [`OverMut`]: crate::traversal::OverMut
    /// [`OverData`]: crate::traversal::OverData
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    /// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
    /// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
    fn iter_mut<'t, 'a, V, M, P, MO>(
        &'t mut self,
        node: &mut NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut,
        'a: 't;

    fn into_iter_with_storage<'a, V, M, P, MO>(
        node_mut: NodeMut<'a, V, M, P, MO>,
        storage: impl SoM<Self::Storage<V>>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: Over;

    fn into_iter_with_storage_filtered<'a, V, M, P, MO, F>(
        node_mut: NodeMut<'a, V, M, P, MO>,
        storage: impl SoM<Self::Storage<V>>,
        filter: F,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: Over,
        F: Fn(&<O::Enumeration as Enumeration>::Item<NodePtr<V>>) -> bool;

    /// Returns an iterator which:
    ///
    /// * traverses all nodes including the `node` and its descendants; i.e.,
    ///   all nodes of the subtree rooted at the given `node`,
    /// * removes the traversed nodes from the tree, and
    /// * yields their values.
    ///
    /// Once the iterator is consumed, the tree will be missing the subtree rooted at the given `node`.
    /// If the given node is the root of the tree, the tree will be empty after this call.
    ///
    /// The order of visited nodes depends on the internal walk strategy of the traverser; depth-first and
    /// breadth-first are the most well-known traversals.
    ///
    /// Typically, the `iter` or `iter_mut` or `into_iter` call of a traverser does not require any memory allocation.
    ///
    /// # Yields
    ///
    /// The return value of the iterations depend on the second generic parameter of the traverser which implements
    /// the [`OverMut`] trait. The following is the complete list of implementations and the corresponding item type
    /// of the created iterators. For any `Over` implementation, the corresponding traverser can be created by using
    /// the `Default::default()` function; however, it is often more convenient to use the [`Traversal`] type.
    /// The last column of the table demonstrates how to create different traverser types; where the depth first or dfs
    /// is replaceable with any available traversal strategy such as `bfs` or `post_order`.
    ///
    /// Importantly note that, since the created iterators remove the nodes from the tree, the "data" below represents
    /// the owned data taken out of the corresponding node, and hence, out of the tree.
    ///
    /// | Over | Yields | Depth First Example |
    /// |---|---|---|
    /// | [`OverData`] | data | `Traversal.dfs()` |
    /// | [`OverDepthData`] | (depth, data) | `Traversal.dfs().with_depth()` |
    /// | [`OverSiblingIdxData`] | (sibling_idx, data) | `Traversal.dfs().with_sibling_idx()` |
    /// | [`OverDepthSiblingIdxData`] | (depth, sibling_idx, data) | `Traversal.with_depth().with_sibling_idx()` |
    ///
    /// [`Traversal`]: crate::traversal::Traversal
    /// [`OverMut`]: crate::traversal::OverMut
    /// [`OverData`]: crate::traversal::OverData
    /// [`OverDepthData`]: crate::traversal::OverDepthData
    /// [`OverSiblingIdxData`]: crate::traversal::OverSiblingIdxData
    /// [`OverDepthSiblingIdxData`]: crate::traversal::OverDepthSiblingIdxData
    #[allow(clippy::wrong_self_convention)]
    fn into_iter<'a, V, M, P, MO>(
        &'a mut self,
        node: NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: OverMut;

    // provided

    fn iter_ptr_with_owned_storage<'t, V>(
        node_ptr: NodePtr<V>,
    ) -> impl Iterator<Item = <O::Enumeration as Enumeration>::Item<NodePtr<V>>>
    where
        V: TreeVariant + 't,
        <Self as TraverserCore<O>>::Storage<V>: 't,
    {
        Self::iter_ptr_with_storage(node_ptr, Self::Storage::default())
    }

    fn iter_with_owned_storage<'t, 'a, V, M, P>(
        node: &impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        Self::Storage<V>: 't,
        'a: 't,
    {
        Self::iter_with_storage(node, Self::Storage::default())
    }

    fn iter_mut_with_owned_storage<'t, 'a, V, M, P, MO>(
        node: &mut NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>> + 't
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        Self::Storage<V>: 't,
        MO: NodeMutOrientation,
        O: OverMut,
        'a: 't,
    {
        Self::iter_mut_with_storage(node_mut, Self::Storage::default())
    }

    fn into_iter_with_owned_storage<'a, V, M, P, MO>(
        node_mut: NodeMut<'a, V, M, P, MO>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
        O: Over,
    {
        Self::into_iter_with_storage(node_mut, Self::Storage::default())
    }
}
