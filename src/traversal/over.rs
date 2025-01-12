use super::breadth_first::BreadthFirstEnumeration;
use super::depth_first::DepthFirstEnumeration;
use super::node_item::NodeItem;
use super::post_order::PostOrderEnumeration;
use crate::memory::{Auto, MemoryPolicy};
use crate::pinned_storage::{PinnedStorage, SplitRecursive};
use crate::traversal::enumeration::Enumeration;
use crate::traversal::enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val};
use crate::{Node, TreeVariant};
use orx_selfref_col::NodePtr;

pub type OverItem<'a, V, O, M = Auto, P = SplitRecursive> =
    <<O as Over>::Enumeration as Enumeration>::Item<<O as Over>::NodeItem<'a, V, M, P>>;

/// Type that defines the type of the items that iterators created by a traverser such as the [`Dfs`] or [`PostOrder`].
///
/// [`Dfs`]: crate::traversal::Dfs
/// [`PostOrder`]: crate::traversal::PostOrder
pub trait Over: 'static {
    /// Enumeration of the traversal, which might be only the node item; or it might include one or both of the
    /// depth and sibling index.
    type Enumeration: Enumeration
        + PostOrderEnumeration
        + DepthFirstEnumeration
        + BreadthFirstEnumeration;

    /// Part of the iterator item which only depends on the node's internal data.
    type NodeItem<'a, V, M, P>: NodeItem<'a, V, M, P>
    where
        V: TreeVariant,
        M: MemoryPolicy,
        P: PinnedStorage,
        V: 'a,
        Self: 'a;

    /// Transformed version of the over item where it yields data rather than Node.
    type IntoOverData: Over;

    /// Transformed version of the over item where it yields Node rather than data.
    type IntoOverNode: Over;

    /// Transformed version of the over item where it yields
    ///
    /// * (depth, x) rather than x, or
    /// * (depth, sibling_idx, x) rather than (sibling_idx, x)
    ///
    /// where x might be data or Node.
    type IntoWithDepth: Over;

    /// Transformed version of the over item where it yields
    ///
    /// * (sibling_idx, x) rather than x, or
    /// * (depth, sibling_idx, x) rather than (depth, x)
    ///
    /// where x might be data or Node.
    type IntoWithSiblingIdx: Over;
}

// val

/// Yields the data of the nodes; i.e., [`data`] and [`data_mut`].
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverData;

impl Over for OverData {
    type Enumeration = Val;

    type NodeItem<'a, V, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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

impl Over for OverNode {
    type Enumeration = Val;

    type NodeItem<'a, V, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = Self;
    type IntoWithDepth = OverDepthNode;
    type IntoWithSiblingIdx = OverSiblingIdxNode;
}

pub(crate) struct OverPtr;

impl Over for OverPtr {
    type Enumeration = Val;

    type NodeItem<'a, V, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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

impl Over for OverDepthData {
    type Enumeration = DepthVal;

    type NodeItem<'a, V, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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

impl Over for OverDepthNode {
    type Enumeration = DepthVal;

    type NodeItem<'a, V, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;

    type IntoOverData = OverDepthData;
    type IntoOverNode = Self;
    type IntoWithDepth = OverDepthNode;
    type IntoWithSiblingIdx = OverDepthSiblingIdxNode;
}

pub(crate) struct OverDepthPtr;

impl Over for OverDepthPtr {
    type Enumeration = Val;

    type NodeItem<'a, V, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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
/// This is because the root is the only sibling in the subtree that the traversal considers.
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverSiblingIdxData;

impl Over for OverSiblingIdxData {
    type Enumeration = SiblingIdxVal;

    type NodeItem<'a, V, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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
/// This is because the root is the only sibling in the subtree that the traversal considers.
///
/// [`Node`]: crate::Node
pub struct OverSiblingIdxNode;

impl Over for OverSiblingIdxNode {
    type Enumeration = SiblingIdxVal;

    type NodeItem<'a, V, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;

    type IntoOverData = OverSiblingIdxData;
    type IntoOverNode = Self;
    type IntoWithDepth = OverDepthSiblingIdxNode;
    type IntoWithSiblingIdx = OverSiblingIdxNode;
}

pub(crate) struct OverSiblingIdxPtr;

impl Over for OverSiblingIdxPtr {
    type Enumeration = Val;

    type NodeItem<'a, V, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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
/// This is because the root is the only sibling in the subtree that the traversal considers.
///
/// [`data`]: crate::NodeRef::data
/// [`data_mut`]: crate::NodeMut::data_mut
pub struct OverDepthSiblingIdxData;

impl Over for OverDepthSiblingIdxData {
    type Enumeration = DepthSiblingIdxVal;

    type NodeItem<'a, V, M, P>
        = &'a V::Item
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
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
/// This is because the root is the only sibling in the subtree that the traversal considers.
///
/// [`Node`]: crate::Node
pub struct OverDepthSiblingIdxNode;

impl Over for OverDepthSiblingIdxNode {
    type Enumeration = DepthSiblingIdxVal;

    type NodeItem<'a, V, M, P>
        = Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;

    type IntoOverData = OverDepthSiblingIdxData;
    type IntoOverNode = Self;
    type IntoWithDepth = Self;
    type IntoWithSiblingIdx = Self;
}
pub(crate) struct OverDepthSiblingIdxPtr;

impl Over for OverDepthSiblingIdxPtr {
    type Enumeration = Val;

    type NodeItem<'a, V, M, P>
        = NodePtr<V>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        V: TreeVariant + 'a,
        Self: 'a;

    type IntoOverData = OverData;
    type IntoOverNode = OverNode;
    type IntoWithDepth = Self;
    type IntoWithSiblingIdx = Self;
}
