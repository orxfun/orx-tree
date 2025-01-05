use crate::{
    node_ref::NodeRefCore,
    traversal::{
        depth_first::{dfs::Dfs, iter_ref::DfsIterRef, DfsIterPtr},
        node_item::NodeItem,
        over::OverData,
        DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Traverser, Val,
    },
    AsTreeNode, Dyn, DynTree, Node, NodeRef,
};
use alloc::vec::Vec;
use orx_selfref_col::{NodePtr, Variant};

/// ```
///      1
///     ╱ ╲
///    ╱   ╲
///   2     3
///  ╱ ╲   ╱ ╲
/// 4   5 6   7
/// |     |  ╱ ╲
/// 8     9 10  11
/// ```
fn tree() -> DynTree<i32> {
    let mut tree = DynTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();
    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree).push(8);

    let mut n3 = id3.node_mut(&mut tree);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree).push(9);
    id7.node_mut(&mut tree).extend([10, 11]);

    tree
}

fn dfs_iter_for<'a, D>()
where
    D: NodeItem<'a, Dyn<i32>> + 'a,
{
    fn data<'a, D>(iter: impl Iterator<Item = D>) -> Vec<<Dyn<i32> as Variant>::Item>
    where
        D: NodeItem<'a, Dyn<i32>> + 'a,
    {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverData>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(data(iter), [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    // let n3 = root.child(1).unwrap();
    // let ptr = n3.node_ptr().clone();
    // let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    // let iter = DfsIterRef::<_, _, _, Val, _, NodePtr<_>>::from((root.col(), iter));
    // assert_eq!(data(iter), [3, 6, 9, 7, 10, 11]);

    // let n7 = n3.child(1).unwrap();
    // let ptr = n7.node_ptr().clone();
    // let iter = DfsIterPtr::<_, Val, _>::from((stack, ptr));
    // let iter = DfsIterRef::<_, _, _, Val, _, NodePtr<_>>::from((root.col(), iter));
    // assert_eq!(data(iter), [7, 10, 11]);
}
