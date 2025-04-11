use crate::{
    Dyn, DynTree, NodeRef,
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        depth_first::{iter_ptr::DfsIterPtr, iter_ref::DfsIterRef},
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
        node_item::NodeItem,
        over::{Over, OverData, OverNode, OverPtr},
    },
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
    let mut tree = DynTree::new(1);

    let mut root = tree.root_mut();
    let [id2, id3] = root.push_children([2, 3]);

    let mut n2 = tree.node_mut(&id2);
    let [id4, _] = n2.push_children([4, 5]);

    tree.node_mut(&id4).push_child(8);

    let mut n3 = tree.node_mut(&id3);
    let [id6, id7] = n3.push_children([6, 7]);

    tree.node_mut(&id6).push_child(9);
    tree.node_mut(&id7).push_children([10, 11]);

    tree
}

#[test]
fn dfs_iter_ref_empty() {
    let tree = DynTree::empty();
    let iter = DfsIterPtr::<Dyn<i32>, Val>::default();
    let mut iter = DfsIterRef::<_, Auto, SplitRecursive, Val, _, NodePtr<_>>::from((&tree.0, iter));
    assert_eq!(iter.next(), None);
}

type Item<'a, O> = <O as Over>::NodeItem<'a, Dyn<i32>, Auto, SplitRecursive>;

fn dfs_iter_for<O: Over>() {
    fn data<'a, O: Over>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = DfsIterRef::<_, _, _, Val, _, Item<'_, O>>::from((root.col(), iter));
    assert_eq!(data::<'_, O>(iter), [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    let n3 = root.get_child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = DfsIterRef::<_, _, _, Val, _, Item<'_, O>>::from((root.col(), iter));
    assert_eq!(data::<'_, O>(iter), [3, 6, 9, 7, 10, 11]);

    let n7 = n3.get_child(1).unwrap();
    let ptr = n7.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((stack, ptr));
    let iter = DfsIterRef::<_, _, _, Val, _, Item<'_, O>>::from((root.col(), iter));
    assert_eq!(data::<'_, O>(iter), [7, 10, 11]);
}

#[test]
fn dfs_iter_ref_ptr() {
    dfs_iter_for::<OverPtr>();
}

#[test]
fn dfs_iter_ref_val() {
    dfs_iter_for::<OverNode>();
}

#[test]
fn dfs_iter_ref_node() {
    dfs_iter_for::<OverData>();
}

#[test]
fn dfs_iter_ref_depth() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, DepthVal, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]
    );

    let n3 = root.get_child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = DfsIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, DepthVal, _, &i32>::from((root.col(), iter));
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 1, 2, 1, 2, 2]);
}

#[test]
fn dfs_iter_ref_sibling() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter =
        DfsIterRef::<_, Auto, SplitRecursive, SiblingIdxVal, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]
    );

    let n3 = root.get_child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = DfsIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter =
        DfsIterRef::<_, Auto, SplitRecursive, SiblingIdxVal, _, &i32>::from((root.col(), iter));
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 0, 1, 0, 1]);
}

#[test]
fn dfs_iter_ref_depth_sibling() {
    let tree = tree();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _, &i32>::from((
        root.col(),
        iter,
    ));
    assert_eq!(
        iter.clone().map(|x| x.0).collect::<Vec<_>>(),
        [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]
    );

    assert_eq!(
        iter.map(|x| x.1).collect::<Vec<_>>(),
        [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]
    );

    let n3 = root.get_child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = DfsIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _, &i32>::from((
        root.col(),
        iter,
    ));
    assert_eq!(
        iter.clone().map(|x| x.0).collect::<Vec<_>>(),
        [0, 1, 2, 1, 2, 2]
    );

    assert_eq!(iter.map(|x| x.1).collect::<Vec<_>>(), [0, 0, 0, 1, 0, 1]);
}
