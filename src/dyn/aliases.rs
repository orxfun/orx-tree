use super::Dyn;
use crate::{
    tree::{DefaultMemory, DefaultPinVec},
    Node, Tree,
};

/// A dynamic tree where each of the nodes might have any number of child nodes.
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
/// DynTree<T>     ==> Tree<Dyn<T>>
/// DynNode<'a, T> ==> Node<'a, Dyn<T>>
/// ```
///
/// *Type aliases with default pinned vector storage (recommended):*
///
/// ```ignore
/// DynTree<T, M>     ==> Tree<Dyn<T>, M>
/// DynNode<'a, T, M> ==> Node<'a, Dyn<T>, M>
/// ```
///
/// *The most general type aliases:*
///
/// ```ignore
/// DynTree<T, M, P>     ==> Tree<Dyn<T>, M, P>
/// DynNode<'a, T, M, P> ==> Node<'a, Dyn<T>, M, P>
/// ```
///
/// TODO: documentation & examples here
pub type DynTree<T, M = DefaultMemory<Dyn<T>>, P = DefaultPinVec<Dyn<T>>> = Tree<Dyn<T>, M, P>;

/// Node of a [`DynTree`].
pub type DynNode<'a, T, M = DefaultMemory<Dyn<T>>, P = DefaultPinVec<Dyn<T>>> =
    Node<'a, Dyn<T>, M, P>;
