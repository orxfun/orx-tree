use super::Dyn;
use crate::{
    memory::Auto,
    pinned_storage::{PinnedStorage, SplitRecursive},
    MemoryPolicy, Node, Tree,
};

/// A dynamic tree where each of the nodes might have any number of child nodes.
///
/// # Examples
///
///
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
/// Please see the relevant documentations of [`NodeIdx`] and [`MemoryPolicy`].
///
/// [`NodeIdx`]: crate::NodeIdx
/// [`MemoryPolicy`]: crate::MemoryPolicy
///
/// *The most general type aliases, by explicitly setting a PinnedVec*
///
/// ```ignore
/// DynTree<T, M, P>     ==> Tree<Dyn<T>, M, P>
/// DynNode<'a, T, M, P> ==> Node<'a, Dyn<T>, M, P>
/// ```
pub type DynTree<T, M = Auto, P = SplitRecursive> = Tree<Dyn<T>, M, P>;

/// Node of a [`DynTree`].
pub type DynNode<'a, T, M = Auto, P = SplitRecursive> = Node<'a, Dyn<T>, M, P>;

#[test]
fn abc() {
    use crate::*;
    use alloc::vec::Vec;

    // # A. BUILDING A TREE

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11

    let mut tree = DynTree::new(1i32);

    let mut root = tree.root_mut();
    let [id2, id3] = root.push_children([2, 3]);
    let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
    tree.node_mut(&id4).push_child(8);
    let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
    tree.node_mut(&id6).push_child(9);
    tree.node_mut(&id7).push_children([10, 11]);
}
