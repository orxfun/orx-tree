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
        #[inline(always)]
        fn data_of<V>(node_ptr: NodePtr<V>) -> V::Item
        where
            V: TreeVariant,
            V::Item: Clone,
        {
            (unsafe { &*node_ptr.ptr() })
                .data()
                .expect("node is active")
                .clone()
        }

        match self.get_root() {
            None => Self::empty(),
            Some(root) => {
                let mut dfs = Dfs::<OverDepthPtr>::new();
                let mut iter = root.walk_with(&mut dfs);
                let (mut current_depth, src_ptr) = iter.next().expect("tree is not empty");

                let mut tree = Self::new(data_of(src_ptr));
                let mut dst = tree.root_mut();

                for (depth, ptr) in iter {
                    match depth > current_depth {
                        true => debug_assert_eq!(depth, current_depth + 1, "dfs error in clone"),
                        false => {
                            let num_parent_moves = current_depth - depth + 1;
                            for _ in 0..num_parent_moves {
                                dst = dst.into_parent_mut().expect("in bounds");
                            }
                        }
                    }
                    let [idx] = dst.grow([data_of(ptr)]);
                    dst = tree.node_mut(&idx);
                    current_depth = depth;
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
