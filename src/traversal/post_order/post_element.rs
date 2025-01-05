use super::states::States;
use crate::traversal::{DepthSiblingIdxVal, DepthVal, Element, SiblingIdxVal, Val};
use crate::TreeVariant;

pub trait PostOrderElement: Element {
    fn create_post_item<D, V>(node_value: D, depth: usize, states: &States<V>) -> Self::Item<D>
    where
        V: TreeVariant;
}

// impl

impl PostOrderElement for Val {
    fn create_post_item<D, V>(node_value: D, _: usize, _: &States<V>) -> Self::Item<D>
    where
        V: TreeVariant,
    {
        node_value
    }
}

impl PostOrderElement for DepthVal {
    fn create_post_item<D, V>(node_value: D, depth: usize, _: &States<V>) -> Self::Item<D>
    where
        V: TreeVariant,
    {
        (depth, node_value)
    }
}

impl PostOrderElement for SiblingIdxVal {
    fn create_post_item<D, V>(node_value: D, depth: usize, states: &States<V>) -> Self::Item<D>
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

impl PostOrderElement for DepthSiblingIdxVal {
    fn create_post_item<D, V>(node_value: D, depth: usize, states: &States<V>) -> Self::Item<D>
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
