use super::tree_variant::TreeVariant;
use crate::{mutations::insert::Insertion, tree::Tree};
use orx_selfref_col::{
    MemoryReclaimOnThreshold, NodeDataLazyClose, NodeRefSingle, NodeRefsArray, Variant,
};

pub type Binary = Dary<2>;
pub type Ternary = Dary<3>;

pub struct Dary<const N: usize>;

impl<'a, const N: usize, T: 'a> Variant<'a, T> for Dary<N> {
    type Storage = NodeDataLazyClose<T>;
    type MemoryReclaim = MemoryReclaimOnThreshold<2>;
    type Prev = NodeRefSingle<'a, Self, T>;
    type Next = NodeRefsArray<'a, N, Self, T>;
    type Ends = NodeRefSingle<'a, Self, T>;
}

impl<'a, const N: usize, T: 'a> TreeVariant<'a, T> for Dary<N> {
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
