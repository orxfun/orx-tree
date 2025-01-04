use super::states::States;
use crate::traversal::node_item::NodeItem;
use crate::traversal::{DepthSiblingIdxVal, DepthVal, Element, SiblingIdxVal, Val};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait PostOrderElement: Element {
    fn create_post_item_from_state<'a, V, M, P, E>(
        col: &'a SelfRefCol<V, M, P>,
        node_ptr: NodePtr<V>,
        depth: usize,
        states: &States<V>,
    ) -> Self::Item<E>
    where
        V: TreeVariant,
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
        E: NodeItem<'a, V, M, P>,
    {
        Self::create_post_item(E::from_ptr(col, node_ptr), depth, states)
    }

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
