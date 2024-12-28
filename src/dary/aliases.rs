use super::Dary;
use crate::{
    tree::{DefaultMemory, DefaultPinVec},
    Tree,
};

/// A d-ary tree where each of the nodes might have at most `D` children.
///
/// TODO: documentation & examples here
pub type DaryTree<const D: usize, T, M = DefaultMemory<Dary<D, T>>, P = DefaultPinVec<Dary<D, T>>> =
    Tree<Dary<D, T>, M, P>;

/// A binary tree.
///
/// TODO: documentation & examples here
pub type BinaryTree<T, M = DefaultMemory<Dary<2, T>>, P = DefaultPinVec<Dary<2, T>>> =
    DaryTree<2, T, M, P>;
