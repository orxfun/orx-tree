use super::{NodeValue, QueueElement};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Core iterator return type kind.
pub trait IterKindCore<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    /// Intermediate element that is enqueued & dequeued throughout the iteration.
    type QueueElement: QueueElement<V>;

    /// Part of the return value that is extracted from the node.
    type ValueFromNode: NodeValue<'a, V, M, P>;

    /// Element type of the iterator; i.e., `Iterator::Item`.
    type YieldElement;

    /// Mutable element type of the iterator; i.e., `Iterator::Item`.
    type YieldElementMut;

    /// Creates children from the current parent.
    fn children(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a;

    /// Creates children from the current parent in reverse order.
    fn children_rev(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a;

    /// Creates the element to be yield, or the iterator item, from the queue element.
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElement;

    /// Creates the mutable element to be yield, or the iterator item, from the queue element.
    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElementMut;
}

/// Defines the return element or item of the iterator over the tree.
pub trait IterOver {
    /// Core iteration kind.
    type IterKind<'a, V, M, P>: IterKindCore<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

/// Defines the return element or item of the mutable iterator over the tree.
pub trait IterMutOver: IterOver {}

// helpers

/// # Safety
///
/// It is safe to use this method within this file which guarantees that:
/// * node_ptr points to an active node with a valid pointer.
/// * created node reference is not leaked by the next method of the iterator.
///
/// node_ptr points to a node belonging to the collection `col`.
/// The collection is referenced for the lifetime of 'a; therefore, the
/// node that the pointer is pointing at will be valid for at least 'a.
#[inline(always)]
pub(super) fn node<'a, V: TreeVariant>(node_ptr: &NodePtr<V>) -> &'a N<V> {
    unsafe { &*node_ptr.ptr() }
}

#[inline(always)]
pub(super) fn node_mut<'a, V: TreeVariant>(node_ptr: &NodePtr<V>) -> &'a mut N<V> {
    unsafe { &mut *node_ptr.ptr_mut() }
}
