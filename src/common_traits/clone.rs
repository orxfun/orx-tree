use crate::{
    helpers::Col,
    iter::AncestorsIterPtr,
    memory::{Auto, MemoryPolicy},
    node_ref::NodeRefCore,
    pinned_storage::{PinnedStorage, SplitRecursive},
    traversal::{over::OverDepthPtr, traverser_core::TraverserCore},
    tree_node_idx::INVALID_IDX_ERROR,
    tree_variant::RefsChildren,
    Dfs, Node, NodeIdx, NodeMut, NodeRef, NodeSwapError, Traversal, Traverser, Tree, TreeVariant,
};
use core::cmp::Ordering::*;
use orx_selfref_col::{NodeIdxError, NodePtr, RefsSingle};

impl<V, M, P> Clone for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
    V::Item: Clone,
{
    fn clone(&self) -> Self {
        match self.get_root() {
            None => Self::empty(),
            Some(root) => {
                let mut traverser = Dfs::<OverDepthPtr>::new();
                let mut tree = Self::new(root.data().clone());

                for child in root.children() {
                    tree.root_mut().push_tree_with(&child, &mut traverser);
                }

                tree
            }
        }
    }
}

#[test]
fn xyz() {
    use crate::*;
    use alloc::vec::Vec;

    //      0
    //     ╱ ╲
    //    ╱   ╲
    //   1     2
    //  ╱ ╲   ╱ ╲
    // 3   4 5   6
    // |     |  ╱ ╲
    // 7     8 9  10

    let mut tree = DynTree::<i32>::new(0);

    let mut root = tree.root_mut();
    let [id1, id2] = root.grow([1, 2]);

    let mut n1 = tree.node_mut(&id1);
    let [id3, _] = n1.grow([3, 4]);

    tree.node_mut(&id3).push(7);

    let mut n2 = tree.node_mut(&id2);
    let [id5, id6] = n2.grow([5, 6]);

    tree.node_mut(&id5).push(8);
    tree.node_mut(&id6).extend([9, 10]);

    let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    // clone the entire tree

    let mut t = Bfs::default().over_nodes();

    let clone = tree.clone();
    let bfs: Vec<_> = clone.root().walk_with(&mut t).map(|x| *x.data()).collect();
    assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);

    let indices: Vec<_> = clone.root().indices_with(&mut t).collect();

    // indices are valid only for their trees

    assert_eq!(tree.get_node(&indices[2]), None);
    assert_eq!(tree.try_node(&indices[2]), Err(NodeIdxError::OutOfBounds));

    assert_eq!(clone.get_node(&id2), None);
    assert_eq!(clone.try_node(&id2), Err(NodeIdxError::OutOfBounds));

    assert_eq!(clone.node(&indices[2]).data(), &2);
}
