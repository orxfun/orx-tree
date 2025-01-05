use super::enumeration::Enumeration;

#[derive(Clone)]
pub struct Val;
impl Enumeration for Val {
    type Item<D> = D;

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        map(element)
    }
}

#[derive(Clone)]
pub struct DepthVal;
impl Enumeration for DepthVal {
    type Item<D> = (usize, D);

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, map(element.1))
    }
}

#[derive(Clone)]
pub struct SiblingIdxVal;
impl Enumeration for SiblingIdxVal {
    type Item<D> = (usize, D);

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, map(element.1))
    }
}

#[derive(Clone)]
pub struct DepthSiblingIdxVal;
impl Enumeration for DepthSiblingIdxVal {
    type Item<D> = (usize, usize, D);

    #[inline(always)]
    fn map_node_data<D, M, E>(element: Self::Item<D>, map: M) -> Self::Item<E>
    where
        M: FnOnce(D) -> E,
    {
        (element.0, element.1, map(element.2))
    }
}
