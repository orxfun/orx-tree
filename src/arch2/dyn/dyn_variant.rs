use crate::tree_variant::SealedVariant;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{Node, NodePtr, RefsSingle, RefsVec, Variant};

/// A dynamic tree where each of the nodes might have any number of nodes.
///
/// The [`Dary`] trees and [`Dyn`] trees differ by the following:
/// * Children of [`Dary`] trees are stored as a fixed capacity array.
///   Therefore, we need to know an upper bound on the number of children.
///   As a con, all child nodes are stored directly without requiring any indirection.
///   Therefore, it is preferable in cases where we know `D`,
///   such as in binary, tertiary, etc., trees.
/// * Children of [`Dyn`] trees are stored as a dynamic size vector.
///   Therefore, we do not need to define an upper bound on the number of children,
///   it can grow as long as the vector can grow.
///   As a tradeoff, although they are efficient, vectors add one level of indirection.
///   Therefore, it is preferable when we require the flexibility in size.
///
/// [`Dary`]: crate::Dary
/// [`Dyn`]: crate::Dyn
pub struct Dyn<T> {
    p: PhantomData<T>,
}

impl<T> Variant for Dyn<T> {
    type Item = T;

    type Prev = RefsSingle<Self>;

    type Next = RefsVec<Self>;

    type Ends = RefsSingle<Self>;
}

impl<T> SealedVariant for Dyn<T> {
    fn occupied_ptr_iter<P>(&self) -> impl Iterator<Item = NodePtr<Self>>
    where
        P: PinnedVec<Node<Self>>,
    {
        core::iter::empty()
    }
}
