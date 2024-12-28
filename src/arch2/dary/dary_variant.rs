use core::marker::PhantomData;
use orx_selfref_col::{RefsArray, RefsSingle, Variant};

/// A d-ary tree where each of the nodes might have 0, 1, .., D-1 or D child nodes.
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
pub struct Dary<const D: usize, T> {
    p: PhantomData<T>,
}

impl<const D: usize, T> Variant for Dary<D, T> {
    type Item = T;

    type Prev = RefsSingle<Self>;

    type Next = RefsArray<D, Self>;

    type Ends = RefsSingle<Self>;
}
