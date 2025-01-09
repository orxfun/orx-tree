use super::{states::State, PostOrder};
use crate::{
    traversal::{traverser_core::TraverserCore, Over},
    TreeVariant,
};
use alloc::vec::Vec;

impl<O: Over> TraverserCore<O> for PostOrder<O> {
    type Storage<V>
        = Vec<State<V>>
    where
        V: TreeVariant;
}
