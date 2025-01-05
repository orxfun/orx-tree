use super::{
    iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
    post_enumeration::PostOrderEnumeration, states::States,
};
use crate::{
    helpers::N,
    node_ref::NodeRefCore,
    traversal::{
        over::{Over, OverItem},
        over_mut::{OverItemMut, OverMut},
        traverser_mut::TraverserMut,
        Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

pub struct PostOrder<V>
where
    V: TreeVariant,
{
    stack: States<V>,
}
