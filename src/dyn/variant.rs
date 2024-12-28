use super::reclaimer::DynReclaimer;
use crate::TreeVariant;
use core::marker::PhantomData;
use orx_selfref_col::{RefsSingle, RefsVec, Variant};

/// A dynamic tree where each of the nodes might have any number of child nodes.
pub struct Dyn<T> {
    p: PhantomData<T>,
}

impl<T> Variant for Dyn<T> {
    type Item = T;

    type Prev = RefsSingle<Self>;

    type Next = RefsVec<Self>;

    type Ends = RefsSingle<Self>;
}

impl<T> TreeVariant for Dyn<T> {
    type Reclaimer = DynReclaimer;

    type Children = RefsVec<Self>;
}
