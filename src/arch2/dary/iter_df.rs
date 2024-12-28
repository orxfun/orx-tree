use super::Dary;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{CoreCol, Node};

pub struct IterDepthFirst<'a, const D: usize, T, P>
where
    P: PinnedVec<Node<Dary<D, T>>>,
{
    col: &'a CoreCol<Dary<D, T>, P>,
    curr_child_idx: usize,
}
