use super::Dary;
use crate::{memory::Auto, pinned_storage::SplitRecursive, Node, Tree};

/// A binary tree where each of the nodes might have at most 2 children.
pub type Binary<T> = Dary<2, T>;

/// A d-ary tree where each of the nodes might have at most `D` children.
///
/// # Type Aliases and Generic Parameters
///
/// Below is the list of pairs of tree & node type aliases from the simplest to the most complex.
///
/// Note that the generic parameter `P` can almost always be omitted since the default storage is almost always preferable.
///
/// Generic parameter `M` can also be omitted in most cases to use the default auto reclaim policy.
/// Therefore, we can use the simplest type signatures.
/// However, in certain situations it is preferable to use the *never* reclaim policy which guarantees that the node indices
/// will always remain valid.
///
/// TODO: see also memory documentation
///
/// *Type aliases with default pinned vector storage and default memory reclaim policy:*
///
/// ```ignore
/// DaryTree<D, T>     ==> Tree<Dary<D, T>>
/// DaryNode<'a, D, T> ==> Node<'a, Dary<D, T>>
/// ```
///
/// *Type aliases with default pinned vector storage (recommended):*
///
/// ```ignore
/// DaryTree<D, T, M>     ==> Tree<Dary<D, T>, M>
/// DaryNode<'a, D, T, M> ==> Node<'a, Dary<D, T>, M>
/// ```
///
/// *The most general type aliases:*
///
/// ```ignore
/// DaryTree<D, T, M, P>     ==> Tree<Dary<D, T>, M, P>
/// DaryNode<'a, D, T, M, P> ==> Node<'a, Dary<D, T>, M, P>
/// ```
///
/// TODO: documentation & examples here
pub type DaryTree<const D: usize, T, M = Auto, P = SplitRecursive> = Tree<Dary<D, T>, M, P>;

/// A binary tree where each node might have 0, 1 or 2 children.
///
/// # Type Aliases and Generic Parameters
///
/// Below is the list of pairs of tree & node type aliases from the simplest to the most complex.
///
/// Note that the generic parameter `P` can almost always be omitted since the default storage is almost always preferable.
///
/// Generic parameter `M` can also be omitted in most cases to use the default auto reclaim policy.
/// Therefore, we can use the simplest type signatures.
/// However, in certain situations it is preferable to use the *never* reclaim policy which guarantees that the node indices
/// will always remain valid.
///
/// TODO: see also memory documentation
///
/// *Type aliases with default pinned vector storage and default memory reclaim policy:*
///
/// ```ignore
/// BinaryTree<T>     ==> Tree<Dary<2, T>>
/// BinaryNode<'a, T> ==> Node<'a, Dary<2, T>>
/// ```
///
/// *Type aliases with default pinned vector storage (recommended):*
///
/// ```ignore
/// BinaryTree<T, M>     ==> Tree<Dary<2, T>, M>
/// BinaryNode<'a, T, M> ==> Node<'a, Dary<2, T>, M>
/// ```
///
/// *The most general type aliases:*
///
/// ```ignore
/// BinaryTree<T, M, P>     ==> Tree<Dary<2, T>, M, P>
/// BinaryNode<'a, T, M, P> ==> Node<'a, Dary<2, T>, M, P>
/// ```
///
/// TODO: documentation & examples here
pub type BinaryTree<T, M = Auto, P = SplitRecursive> = Tree<Binary<T>, M, P>;

// nodes

/// Node of a [`DaryTree`].
pub type DaryNode<'a, const D: usize, T, M = Auto, P = SplitRecursive> = Node<'a, Dary<D, T>, M, P>;

/// Node of a [`BinaryTree`].
pub type BinaryNode<'a, T, M = Auto, P = SplitRecursive> = Node<'a, Dary<2, T>, M, P>;
