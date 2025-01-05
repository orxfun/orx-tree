use super::Element;

pub struct Val;
impl Element for Val {
    type Item<D> = D;

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        map(element)
    }
}

pub struct DepthVal;
impl Element for DepthVal {
    type Item<D> = (usize, D);

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, map(element.1))
    }
}

pub struct SiblingIdxVal;
impl Element for SiblingIdxVal {
    type Item<D> = (usize, D);

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, map(element.1))
    }
}

pub struct DepthSiblingIdxVal;
impl Element for DepthSiblingIdxVal {
    type Item<D> = (usize, usize, D);

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, element.1, map(element.2))
    }
}
