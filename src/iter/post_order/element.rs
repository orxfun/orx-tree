use super::{depth_nodes::DepthNodes, enumerator::PostOrderEnumerator};
use crate::{
    helpers::N,
    iter::{
        Enumerator, EnumeratorDepth, EnumeratorDepthSibling, EnumeratorNone, NodeData,
        NodeDataData, NodeDataNode, NodeDataPtr, OverData, OverDepthData, OverDepthNode,
        OverDepthPtr, OverDepthSiblingData, OverDepthSiblingNode, OverDepthSiblingPtr, OverNode,
        OverPtr,
    },
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait PostOrderElement {
    type Enumerator: PostOrderEnumerator;

    type NodeData: NodeData;

    #[inline(always)]
    fn element_ptr<V>(
        node_ptr: NodePtr<V>,
        depth: usize,
        depth_nodes: &DepthNodes<V>,
    ) -> <Self::Enumerator as Enumerator>::Output<NodePtr<V>>
    where
        V: TreeVariant,
    {
        <Self::Enumerator as PostOrderEnumerator>::post_output(node_ptr, depth, depth_nodes)
    }

    #[inline(always)]
    fn element<V, M, P>(
        col: &SelfRefCol<V, M, P>,
        output_ptr: <Self::Enumerator as Enumerator>::Output<NodePtr<V>>,
    ) -> <Self::Enumerator as Enumerator>::Output<<Self::NodeData as NodeData>::Value<'_, V, M, P>>
    where
        V: TreeVariant,
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
    {
        <Self::Enumerator as Enumerator>::from_ptr(output_ptr, |ptr| {
            <Self::NodeData as NodeData>::value(col, ptr)
        })
    }

    #[inline(always)]
    fn element_mut<V, M, P>(
        col: &mut SelfRefCol<V, M, P>,
        output_ptr: <Self::Enumerator as Enumerator>::Output<NodePtr<V>>,
    ) -> <Self::Enumerator as Enumerator>::Output<<Self::NodeData as NodeData>::ValueMut<'_, V, M, P>>
    where
        V: TreeVariant,
        M: MemoryPolicy<V>,
        P: PinnedVec<N<V>>,
    {
        <Self::Enumerator as Enumerator>::from_ptr(output_ptr, |ptr| {
            <Self::NodeData as NodeData>::value_mut(col, ptr)
        })
    }
}

// elements

impl PostOrderElement for OverPtr {
    type Enumerator = EnumeratorNone;
    type NodeData = NodeDataPtr;
}

impl PostOrderElement for OverData {
    type Enumerator = EnumeratorNone;
    type NodeData = NodeDataData;
}

impl PostOrderElement for OverNode {
    type Enumerator = EnumeratorNone;
    type NodeData = NodeDataNode;
}

impl PostOrderElement for OverDepthPtr {
    type Enumerator = EnumeratorDepth;
    type NodeData = NodeDataPtr;
}

impl PostOrderElement for OverDepthData {
    type Enumerator = EnumeratorDepth;
    type NodeData = NodeDataData;
}

impl PostOrderElement for OverDepthNode {
    type Enumerator = EnumeratorDepth;
    type NodeData = NodeDataNode;
}

impl PostOrderElement for OverDepthSiblingPtr {
    type Enumerator = EnumeratorDepthSibling;
    type NodeData = NodeDataPtr;
}

impl PostOrderElement for OverDepthSiblingData {
    type Enumerator = EnumeratorDepthSibling;
    type NodeData = NodeDataData;
}

impl PostOrderElement for OverDepthSiblingNode {
    type Enumerator = EnumeratorDepthSibling;
    type NodeData = NodeDataNode;
}
