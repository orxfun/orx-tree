use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryReclaimer, Node, NodePtr, Variant};

pub trait TreeVariant: Variant {
    type Reclaimer: MemoryReclaimer<Self>;
}

pub(crate) trait SealedVariant: Variant {
    fn occupied_ptr_iter<P>(&self) -> impl Iterator<Item = NodePtr<Self>>
    where
        P: PinnedVec<Node<Self>>;
}
