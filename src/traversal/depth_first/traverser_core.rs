use super::{stack::Item, Dfs};
use crate::{
    traversal::{traverser_core::TraverserCore, Over},
    TreeVariant,
};
use alloc::vec::Vec;

impl<O: Over> TraverserCore<O> for Dfs<O> {
    type Storage<V>
        = Vec<Item<V, O::Enumeration>>
    where
        V: TreeVariant;
}
