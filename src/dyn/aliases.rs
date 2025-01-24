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
/// ```
/// use orx_tree::*;
///
/// // # A. BUILDING A TREE
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲   ╱ ╲
/// // 4   5 6   7
/// // |     |  ╱ ╲
/// // 8     9 10  11
///
/// let mut tree = DynTree::new(1i32);
///
/// let mut root = tree.root_mut();
/// let [id2, id3] = root.push_children([2, 3]);
/// let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
/// let id8 = tree.node_mut(&id4).push_child(8);
/// let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
/// let id9 = tree.node_mut(&id6).push_child(9);
/// tree.node_mut(&id7).push_children([10, 11]);
///
/// // traversals
///
/// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
/// assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
///
/// let dfs: Vec<_> = tree.node(&id3).walk::<Dfs>().copied().collect();
/// assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);
///
/// let post_order: Vec<_> = tree.node(&id3).walk::<PostOrder>().copied().collect();
/// assert_eq!(post_order, [9, 6, 10, 11, 7, 3]);
///
/// let leaves: Vec<_> = tree.root().leaves::<Dfs>().copied().collect();
/// assert_eq!(leaves, [8, 5, 9, 10, 11]);
///
/// let node3 = tree.node(&id3);
/// let paths: Vec<Vec<_>> = node3.paths::<Bfs>().map(|p| p.copied().collect()).collect();
/// assert_eq!(paths, [[9, 6, 3], [10, 7, 3], [11, 7, 3]]);
/// ```
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
/// Please see the relevant documentations of [`NodeIdx`] and [`MemoryPolicy`].
///
/// [`NodeIdx`]: crate::NodeIdx
/// [`MemoryPolicy`]: crate::MemoryPolicy
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
    use alloc::vec;
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
    let id8 = tree.node_mut(&id4).push_child(8);
    let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
    let id9 = tree.node_mut(&id6).push_child(9);
    tree.node_mut(&id7).push_children([10, 11]);

    // B. NODE

    let node4 = tree.node(&id4);

    assert!(!node4.is_leaf());
    assert!(!node4.is_root());
    assert_eq!(node4.depth(), 2);
    assert_eq!(node4.height(), 1);
    assert_eq!(node4.sibling_idx(), 0);
    assert_eq!(node4.parent(), Some(tree.node(&id2)));
    assert_eq!(node4.num_children(), 1);
    assert_eq!(node4.get_child(0), Some(tree.node(&id8)));

    let ancestors: Vec<_> = node4.ancestors().map(|x| *x.data()).collect();
    assert_eq!(ancestors, [4, 2, 1]);

    let new_tree: BinaryTree<_> = node4.clone_as_tree();
    assert_eq!(new_tree.root().data(), &4);
    assert_eq!(new_tree.len(), 2);

    // # B. TRAVERSALS

    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    let dfs: Vec<_> = tree.node(&id3).walk::<Dfs>().copied().collect();
    assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);

    let post_order: Vec<_> = tree.node(&id3).walk::<PostOrder>().copied().collect();
    assert_eq!(post_order, [9, 6, 10, 11, 7, 3]);

    let leaves: Vec<_> = tree.root().leaves::<Dfs>().copied().collect();
    assert_eq!(leaves, [8, 5, 9, 10, 11]);

    let node3 = tree.node(&id3);
    let paths: Vec<Vec<_>> = node3.paths::<Bfs>().map(|p| p.copied().collect()).collect();
    assert_eq!(paths, [[9, 6, 3], [10, 7, 3], [11, 7, 3]]);

    // # D. MUTATIONS - REMOVALS

    let mut tree = tree.into_lazy_reclaim(); // to keep the indices valid

    // > remove a subtree and collect values in the desired traversal order
    let node7 = tree.node_mut(&id7);
    let removed_in_bfs_order: Vec<_> = node7.into_walk::<Bfs>().collect();
    assert_eq!(removed_in_bfs_order, [7, 10, 11]);
    let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(remaining, [1, 2, 3, 4, 5, 6, 8, 9]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱
    // 4   5 6
    // |     |
    // 8     9

    // > take out just one node
    let node6 = tree.node_mut(&id6);
    let taken_out = node6.take_out(); // 6 is removed, 9 moves up
    assert_eq!(taken_out, 6);
    let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(remaining, [1, 2, 3, 4, 5, 9, 8]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱
    // 4   5 9
    // |
    // 8

    // > prune a subtree
    let node2 = tree.node_mut(&id2);
    let taken_out = node2.prune(); // 2 is removed, together with descendants
    assert_eq!(taken_out, 2);
    let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(remaining, [1, 3, 9]);
    //      1
    //       ╲
    //        ╲
    //         3
    //        ╱
    //       9

    // # C. MUTATIONS - ADDING & MOVING SUBTREES

    // > append another tree as a sibling of a node
    let mut other_tree = DynTree::new(2);
    let [id4, _] = other_tree.root_mut().push_children([4, 5]);
    other_tree.node_mut(&id4).push_child(8);

    let id2 = tree
        .node_mut(&id3)
        .push_sibling_tree(Side::Left, other_tree);
    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 4, 5, 9, 8]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱
    // 4   5 9
    // |
    // 8

    // > move a subtree to another location in the same tree
    let node2 = tree.node(&id2);
    let [id4, id5] = [node2.child(0).idx(), node2.child(1).idx()];
    tree.node_mut(&id3)
        .push_child_tree_within(id4.into_subtree_within());
    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 5, 9, 4, 8]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //    ╲   ╱ ╲
    //     5 9   4
    //           |
    //           8

    // > move the subtree back
    tree.node_mut(&id5)
        .push_sibling_tree_within(Side::Left, id4.into_subtree_within());
    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 4, 5, 9, 8]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱
    // 4   5 9
    // |
    // 8

    // > insert a node in between parent & child
    tree.node_mut(&id9).push_parent(6);
    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 8, 9]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱
    // 4   5 6
    // |     |
    // 8     9

    // push a subtree cloned/copied from another tree
    let mut other_tree = DynTree::new(100);
    let id7 = other_tree.root_mut().push_child(7);
    other_tree.node_mut(&id7).push_children([10, 11]);

    let subtree = other_tree.node(&id7).as_cloned_subtree();
    tree.node_mut(&id3).push_child_tree(subtree);

    assert_eq!(other_tree.len(), 4); // unchanged

    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11
}
