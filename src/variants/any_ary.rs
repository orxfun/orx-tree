use super::tree_variant::TreeVariant;
use crate::{mutations::insert::Insertion, tree::Tree};
use orx_selfref_col::{
    MemoryReclaimOnThreshold, NodeDataLazyClose, NodeRefSingle, NodeRefsVec, Variant,
};

pub struct AnyAry;

impl<'a, T: 'a> Variant<'a, T> for AnyAry {
    type Storage = NodeDataLazyClose<T>;
    type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    type Prev = NodeRefSingle<'a, Self, T>;
    type Next = NodeRefsVec<'a, Self, T>;
    type Ends = NodeRefSingle<'a, Self, T>;
}

impl<'a, T: 'a> TreeVariant<'a, T> for AnyAry {
    fn insert(tree: &mut Tree<'a, Self, T>, insertion: Insertion<'a, Self, T>) {
        match insertion {
            Insertion::None => {}
            Insertion::AsParentOf(child) => {
                // asdf
            }
            Insertion::AsChildOf(parent, child_index) => {
                //
            }
        }
        todo!()
    }
}
