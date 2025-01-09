use super::states::State;
use crate::traversal::enumeration::Enumeration;
use crate::traversal::enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val};
use crate::TreeVariant;

pub trait PostOrderEnumeration: Enumeration {
    fn create_post_item<D, V>(node_value: D, depth: usize, states: &[State<V>]) -> Self::Item<D>
    where
        V: TreeVariant;
}

// impl

impl PostOrderEnumeration for Val {
    fn create_post_item<D, V>(node_value: D, _: usize, _: &[State<V>]) -> Self::Item<D>
    where
        V: TreeVariant,
    {
        node_value
    }
}

impl PostOrderEnumeration for DepthVal {
    fn create_post_item<D, V>(node_value: D, depth: usize, _: &[State<V>]) -> Self::Item<D>
    where
        V: TreeVariant,
    {
        (depth, node_value)
    }
}

impl PostOrderEnumeration for SiblingIdxVal {
    fn create_post_item<D, V>(node_value: D, depth: usize, states: &[State<V>]) -> Self::Item<D>
    where
        V: TreeVariant,
    {
        let sibling_idx = match depth {
            0 => 0,
            d => states[d - 1].1,
        };
        (sibling_idx, node_value)
    }
}

impl PostOrderEnumeration for DepthSiblingIdxVal {
    fn create_post_item<D, V>(node_value: D, depth: usize, states: &[State<V>]) -> Self::Item<D>
    where
        V: TreeVariant,
    {
        let sibling_idx = match depth {
            0 => 0,
            d => states[d - 1].1,
        };
        (depth, sibling_idx, node_value)
    }
}
