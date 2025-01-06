use super::depth_nodes::DepthNodes;
use crate::{
    iter::{
        Enumerator, EnumeratorDepth, EnumeratorDepthSibling, EnumeratorNone, EnumeratorSibling,
    },
    TreeVariant,
};

pub trait PostOrderEnumerator: Enumerator {
    fn post_output<V: TreeVariant, Data>(
        data: Data,
        depth: usize,
        depth_nodes: &DepthNodes<V>,
    ) -> Self::Output<Data>;
}

impl PostOrderEnumerator for EnumeratorNone {
    #[inline(always)]
    fn post_output<V: TreeVariant, Data>(
        data: Data,
        _: usize,
        _: &DepthNodes<V>,
    ) -> Self::Output<Data> {
        data
    }
}

impl PostOrderEnumerator for EnumeratorDepth {
    #[inline(always)]
    fn post_output<V: TreeVariant, Data>(
        data: Data,
        depth: usize,
        _: &DepthNodes<V>,
    ) -> Self::Output<Data> {
        (depth, data)
    }
}

impl PostOrderEnumerator for EnumeratorSibling {
    #[inline(always)]
    fn post_output<V: TreeVariant, Data>(
        data: Data,
        depth: usize,
        depth_nodes: &DepthNodes<V>,
    ) -> Self::Output<Data> {
        let sibling_idx = match depth {
            0 => 0,
            d => depth_nodes.get(d - 1).child_idx(),
        };
        (sibling_idx, data)
    }
}

impl PostOrderEnumerator for EnumeratorDepthSibling {
    #[inline(always)]
    fn post_output<V: TreeVariant, Data>(
        data: Data,
        depth: usize,
        depth_nodes: &DepthNodes<V>,
    ) -> Self::Output<Data> {
        let sibling_idx = match depth {
            0 => 0,
            d => depth_nodes.get(d - 1).child_idx(),
        };
        (depth, sibling_idx, data)
    }
}
