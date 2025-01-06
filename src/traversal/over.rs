use super::depth_first::DepthFirstEnumeration;
use super::node_item::NodeItem;
use super::post_order::PostOrderEnumeration;
use crate::traversal::enumeration::Enumeration;
use crate::traversal::enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val};
use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    Node, TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

pub type OverItem<'a, V, O, M = DefaultMemory<V>, P = DefaultPinVec<V>> =
    <<O as Over<V>>::Enumeration as Enumeration>::Item<<O as Over<V>>::NodeItem<'a, M, P>>;

/// Type that defines the type of the items that iterators created by a traverser such as the [`Dfs`] or [`PostOrder`].
///
/// [`Dfs`]: crate::traversal::Dfs
/// [`PostOrder`]: crate::traversal::PostOrder
pub trait Over<V: TreeVariant> {
    /// Enumeration of the traversal, which might be only the node item; or it might include one or both of the
    /// depth and sibling index.
    type Enumeration: Enumeration + PostOrderEnumeration + DepthFirstEnumeration;

    /// Part of the iterator item which only depends on the node's internal data.
    type NodeItem<'a, M, P>: NodeItem<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    /// Transformed version of the over item where it yields data rather than Node.
    type IntoOverData: Over<V>;

    /// Transformed version of the over item where it yields Node rather than data.
    type IntoOverNode: Over<V>;

    /// Transformed version of the over item where it yields
    ///
    /// * (depth, x) rather than x, or
    /// * (depth, sibling_idx, x) rather than (sibling_idx, x)
    ///
    /// where x might be data or Node.
    type IntoWithDepth: Over<V>;

    /// Transformed version of the over item where it yields
    ///
    /// * (sibling_idx, x) rather than x, or
    /// * (depth, sibling_idx, x) rather than (depth, x)
    ///
    /// where x might be data or Node.
    type IntoWithSiblingIdx: Over<V>;
}

// val

/// Yields the data of the nodes; i.e., [`data`] and [`data_mut`].
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverData;

impl<V: TreeVariant> Over<V> for OverData {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = Self;
    type IntoOverNode = OverNode;
    type IntoWithDepth = OverDepthData;
    type IntoWithSiblingIdx = OverSiblingIdxData;
}

/// Yields a reference to the nodes; i.e., [`Node`].
///
/// [`Node`]: crate::Node
pub struct OverNode;

impl<V: TreeVariant> Over<V> for OverNode {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = Self;
    type IntoWithDepth = OverDepthNode;
    type IntoWithSiblingIdx = OverSiblingIdxNode;
}

pub(crate) struct OverPtr;

impl<V: TreeVariant> Over<V> for OverPtr {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = OverNode;
    type IntoWithDepth = OverDepthPtr;
    type IntoWithSiblingIdx = OverSiblingIdxPtr;
}

// depth & val

/// Yields (depth, data) tuple of the nodes; where data might be [`data`] and [`data_mut`].
///
/// The depth is relative to the root of the traversal, rather than the root of the tree.
/// In other words, the depth of the node that the traversal is initiated from will be 0;
/// and depth of its descendants will be evaluated relative to this.
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverDepthData;

impl<V: TreeVariant> Over<V> for OverDepthData {
    type Enumeration = DepthVal;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = Self;
    type IntoOverNode = OverDepthNode;
    type IntoWithDepth = OverDepthData;
    type IntoWithSiblingIdx = OverDepthSiblingIdxData;
}

/// Yields (depth, [`Node`]) tuple of the nodes.
///
/// The depth is relative to the root of the traversal, rather than the root of the tree.
/// In other words, the depth of the node that the traversal is initiated from will be 0;
/// and depth of its descendants will be evaluated relative to this.
///
/// [`Node`]: crate::Node
pub struct OverDepthNode;

impl<V: TreeVariant> Over<V> for OverDepthNode {
    type Enumeration = DepthVal;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverDepthData;
    type IntoOverNode = Self;
    type IntoWithDepth = OverDepthNode;
    type IntoWithSiblingIdx = OverDepthSiblingIdxNode;
}

pub(crate) struct OverDepthPtr;

impl<V: TreeVariant> Over<V> for OverDepthPtr {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = OverNode;
    type IntoWithDepth = OverDepthPtr;
    type IntoWithSiblingIdx = OverDepthSiblingIdxPtr;
}

// sibling & val

/// Yields (sibling_idx, data) tuple of the nodes; where data might be [`data`] and [`data_mut`].
///
/// Sibling indices of all nodes except for the root of the traversal are naturally equal to the sibling
/// indices of the nodes in the tree.
///
/// However, sibling index of the root, or the node that the traversal is initiated from, will be 0.
/// This is because the root is the only sibling in the sub-tree that the traversal considers.
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverSiblingIdxData;

impl<V: TreeVariant> Over<V> for OverSiblingIdxData {
    type Enumeration = SiblingIdxVal;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = Self;
    type IntoOverNode = OverSiblingIdxNode;
    type IntoWithDepth = OverDepthSiblingIdxData;
    type IntoWithSiblingIdx = OverSiblingIdxData;
}

/// Yields (sibling_idx, [`Node`]) tuple of the nodes.
///
/// Sibling indices of all nodes except for the root of the traversal are naturally equal to the sibling
/// indices of the nodes in the tree.
///
/// However, sibling index of the root, or the node that the traversal is initiated from, will be 0.
/// This is because the root is the only sibling in the sub-tree that the traversal considers.
///
/// [`Node`]: crate::Node
pub struct OverSiblingIdxNode;

impl<V: TreeVariant> Over<V> for OverSiblingIdxNode {
    type Enumeration = SiblingIdxVal;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverSiblingIdxData;
    type IntoOverNode = Self;
    type IntoWithDepth = OverDepthSiblingIdxNode;
    type IntoWithSiblingIdx = OverSiblingIdxNode;
}

pub(crate) struct OverSiblingIdxPtr;

impl<V: TreeVariant> Over<V> for OverSiblingIdxPtr {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = OverNode;
    type IntoWithDepth = OverDepthSiblingIdxPtr;
    type IntoWithSiblingIdx = OverSiblingIdxPtr;
}

// depth & sibling & val

/// Yields (depth, sibling_idx, data) tuple of the nodes; where data might be [`data`] and [`data_mut`].
///
/// The depth is relative to the root of the traversal, rather than the root of the tree.
/// In other words, the depth of the node that the traversal is initiated from will be 0;
/// and depth of its descendants will be evaluated relative to this.
///
/// Sibling indices of all nodes except for the root of the traversal are naturally equal to the sibling
/// indices of the nodes in the tree.
///
/// However, sibling index of the root will be 0.
/// This is because the root is the only sibling in the sub-tree that the traversal considers.
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverDepthSiblingIdxData;

impl<V: TreeVariant> Over<V> for OverDepthSiblingIdxData {
    type Enumeration = DepthSiblingIdxVal;

    type NodeItem<'a, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = Self;
    type IntoOverNode = OverDepthSiblingIdxNode;
    type IntoWithDepth = Self;
    type IntoWithSiblingIdx = Self;
}

/// Yields (depth, sibling_idx, [`Node`]) tuple of the nodes.
///
/// The depth is relative to the root of the traversal, rather than the root of the tree.
/// In other words, the depth of the node that the traversal is initiated from will be 0;
/// and depth of its descendants will be evaluated relative to this.
///
/// Sibling indices of all nodes except for the root of the traversal are naturally equal to the sibling
/// indices of the nodes in the tree.
///
/// However, sibling index of the root will be 0.
/// This is because the root is the only sibling in the sub-tree that the traversal considers.
///
/// [`Node`]: crate::Node
pub struct OverDepthSiblingIdxNode;

impl<V: TreeVariant> Over<V> for OverDepthSiblingIdxNode {
    type Enumeration = DepthSiblingIdxVal;

    type NodeItem<'a, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverDepthSiblingIdxData;
    type IntoOverNode = Self;
    type IntoWithDepth = Self;
    type IntoWithSiblingIdx = Self;
}
pub(crate) struct OverDepthSiblingIdxPtr;

impl<V: TreeVariant> Over<V> for OverDepthSiblingIdxPtr {
    type Enumeration = Val;

    type NodeItem<'a, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a,
        V: 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = OverNode;
    type IntoWithDepth = Self;
    type IntoWithSiblingIdx = Self;
}
