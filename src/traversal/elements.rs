use super::{node_item::NodeItem, Element};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub struct Val;
impl Element for Val {
    type Item<D> = D;

    fn from_root<D: Clone>(root: D) -> Self::Item<D> {
        root
    }

    #[inline(always)]
    fn node_value<D: Clone>(element: &Self::Item<D>) -> &D {
        element
    }

    #[inline(always)]
    fn children<D: Clone>(
        _: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        children_data
    }

    #[inline(always)]
    fn map<D: Clone, M, E: Clone>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        map(element)
    }
}

pub struct DepthVal;
impl Element for DepthVal {
    type Item<D> = (usize, D);

    fn from_root<D: Clone>(root: D) -> Self::Item<D> {
        (0, root)
    }

    #[inline(always)]
    fn node_value<D: Clone>(element: &Self::Item<D>) -> &D {
        &element.1
    }

    #[inline(always)]
    fn children<D: Clone>(
        parent: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        let depth = parent.0 + 1;
        children_data.map(move |data| (depth, data))
    }

    #[inline(always)]
    fn map<D: Clone, M, E: Clone>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, map(element.1))
    }
}

pub struct SiblingIdxVal;
impl Element for SiblingIdxVal {
    type Item<D> = (usize, D);

    fn from_root<D: Clone>(root: D) -> Self::Item<D> {
        (0, root)
    }

    #[inline(always)]
    fn node_value<D: Clone>(element: &Self::Item<D>) -> &D {
        &element.1
    }

    #[inline(always)]
    fn children<D: Clone>(
        _: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        children_data
            .enumerate()
            .map(|(sibling_idx, data)| (sibling_idx, data))
    }

    #[inline(always)]
    fn map<D: Clone, M, E: Clone>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, map(element.1))
    }
}

pub struct DepthSiblingIdxVal;
impl Element for DepthSiblingIdxVal {
    type Item<D> = (usize, usize, D);

    fn from_root<D: Clone>(root: D) -> Self::Item<D> {
        (0, 0, root)
    }

    #[inline(always)]
    fn node_value<D: Clone>(element: &Self::Item<D>) -> &D {
        &element.2
    }

    #[inline(always)]
    fn children<D: Clone>(
        parent: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        let depth = parent.0 + 1;
        children_data
            .enumerate()
            .map(move |(sibling_idx, data)| (depth, sibling_idx, data))
    }

    #[inline(always)]
    fn map<D: Clone, M, E: Clone>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, element.1, map(element.2))
    }
}
