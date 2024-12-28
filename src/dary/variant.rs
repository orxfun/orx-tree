use super::reclaimer::DaryReclaimer;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_selfref_col::{RefsArrayLeftMost, RefsSingle, Variant};

/// A dynamic tree where each of the nodes might have any number of child nodes.
pub struct Dary<const D: usize, T> {
    p: PhantomData<T>,
}

impl<const D: usize, T> Variant for Dary<D, T> {
    type Item = T;

    type Prev = RefsSingle<Self>;

    type Next = RefsArrayLeftMost<D, Self>;

    type Ends = RefsSingle<Self>;
}

impl<const D: usize, T> TreeVariant for Dary<D, T> {
    type Reclaimer = DaryReclaimer;

    type Children = RefsArrayLeftMost<D, Self>;
}
