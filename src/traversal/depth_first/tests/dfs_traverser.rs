use crate::{
    traversal::{
        depth_first::{dfs::Dfs, dfs_enumeration::DepthFirstEnumeration},
        node_item::NodeItem,
        over::{
            Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverNode, OverPtr,
            OverSiblingIdxData,
        },
        Traverser, Val,
    },
    tree::{DefaultMemory, DefaultPinVec},
    AsTreeNode, Dyn, DynTree, NodeRef,
};
use alloc::vec::Vec;
use orx_selfref_col::Variant;

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

type Item<'a, O> =
    <O as Over<Dyn<i32>>>::NodeItem<'a, DefaultMemory<Dyn<i32>>, DefaultPinVec<Dyn<i32>>>;

fn dfs_iter_for<O: Over<Dyn<i32>, Enumeration = Val>>()
where
    O::Enumeration: DepthFirstEnumeration,
{
    fn data<'a, O: Over<Dyn<i32>> + 'a>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, O>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(data::<O>(iter), [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(data::<O>(iter), [3, 6, 9, 7, 10, 11]);

    let n7 = n3.child(1).unwrap();
    let iter = traverser.iter(&n7);
    assert_eq!(data::<O>(iter), [7, 10, 11]);
}

#[test]
fn dfs_traverser_ptr() {
    dfs_iter_for::<OverPtr>();
}

#[test]
fn dfs_traverser_val() {
    dfs_iter_for::<OverNode>();
}

#[test]
fn dfs_traverser_node() {
    dfs_iter_for::<OverData>();
}

#[test]
fn dfs_iter_ref_depth() {
    let tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverDepthData>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]
    );

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 1, 2, 1, 2, 2]);
}

#[test]
fn dfs_iter_ref_sibling() {
    let tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverSiblingIdxData>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]
    );

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 0, 1, 0, 1]);
}

#[test]
fn dfs_iter_ref_depth_sibling() {
    let tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverDepthSiblingIdxData>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]
    );
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| x.1).collect::<Vec<_>>(),
        [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]
    );

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 1, 2, 1, 2, 2]);
    let iter = traverser.iter(&n3);
    assert_eq!(iter.map(|x| x.1).collect::<Vec<_>>(), [0, 0, 0, 1, 0, 1]);
}
