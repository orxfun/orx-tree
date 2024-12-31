use super::{StackElement, ValueFromNode};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait IterKindCore<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type StackElement: StackElement<V>;

    type ValueFromNode: ValueFromNode<'a, V, M, P>;

    type YieldElement: Clone;

    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a;

    fn element(
        col: &'a SelfRefCol<V, M, P>,
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement;
}

pub trait IterOver {
    type IterKind<'a, V, M, P>: IterKindCore<'a, V, M, P>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;
}

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
