use super::Dyn;
use crate::{
    tree::{DefaultMemory, DefaultPinVec},
    Tree,
};

/// A dynamic tree where each of the nodes might have any number of child nodes.
///
/// TODO: documentation & examples here
pub type DynTree<T, M = DefaultMemory<Dyn<T>>, P = DefaultPinVec<Dyn<T>>> = Tree<Dyn<T>, M, P>;
