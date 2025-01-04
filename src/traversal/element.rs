pub trait Element {
    type Item<NodeData>;
}

pub struct Val;
impl Element for Val {
    type Item<NodeData> = NodeData;
}

pub struct DepthVal;
impl Element for DepthVal {
    type Item<NodeData> = (usize, NodeData);
}

pub struct SiblingIdxVal;
impl Element for SiblingIdxVal {
    type Item<NodeData> = (usize, NodeData);
}

pub struct DepthSiblingIdxVal;
impl Element for DepthSiblingIdxVal {
    type Item<NodeData> = (usize, usize, NodeData);
}
