use crate::{helpers::N, traversal::Element, NodeMut, NodeRef, TreeVariant};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

pub struct Dfs<V: TreeVariant> {
    phantom: PhantomData<V>,
}
