use crate::traversal::{DepthSiblingIdxVal, DepthVal, Element, SiblingIdxVal, Val};

pub trait DepthFirstElement: Element {
    fn from_root<D>(root: D) -> Self::Item<D>;

    fn node_value<D>(element: &Self::Item<D>) -> &D;

    fn children<D>(
        parent: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>>;
}

// impl

impl DepthFirstElement for Val {
    fn from_root<D>(root: D) -> Self::Item<D> {
        root
    }

    #[inline(always)]
    fn node_value<D>(element: &Self::Item<D>) -> &D {
        element
    }

    #[inline(always)]
    fn children<D>(
        _: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        children_data
    }
}

impl DepthFirstElement for DepthVal {
    fn from_root<D>(root: D) -> Self::Item<D> {
        (0, root)
    }

    #[inline(always)]
    fn node_value<D>(element: &Self::Item<D>) -> &D {
        &element.1
    }

    #[inline(always)]
    fn children<D>(
        parent: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        let depth = parent.0 + 1;
        children_data.map(move |data| (depth, data))
    }
}

impl DepthFirstElement for SiblingIdxVal {
    fn from_root<D>(root: D) -> Self::Item<D> {
        (0, root)
    }

    #[inline(always)]
    fn node_value<D>(element: &Self::Item<D>) -> &D {
        &element.1
    }

    #[inline(always)]
    fn children<D>(
        _: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        children_data
            .enumerate()
            .map(|(sibling_idx, data)| (sibling_idx, data))
    }
}

impl DepthFirstElement for DepthSiblingIdxVal {
    fn from_root<D>(root: D) -> Self::Item<D> {
        (0, 0, root)
    }

    #[inline(always)]
    fn node_value<D>(element: &Self::Item<D>) -> &D {
        &element.2
    }

    #[inline(always)]
    fn children<D>(
        parent: &Self::Item<D>,
        children_data: impl DoubleEndedIterator<Item = D> + ExactSizeIterator,
    ) -> impl DoubleEndedIterator<Item = Self::Item<D>> {
        let depth = parent.0 + 1;
        children_data
            .enumerate()
            .map(move |(sibling_idx, data)| (depth, sibling_idx, data))
    }
}
