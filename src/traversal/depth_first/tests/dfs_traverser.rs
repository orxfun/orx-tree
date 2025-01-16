use crate::{
    memory::Auto,
    pinned_storage::SplitRecursive,
    traversal::{
        depth_first::{dfs_enumeration::DepthFirstEnumeration, traverser::Dfs},
        enumerations::Val,
        node_item::NodeItem,
        over::{
            Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverNode, OverPtr,
            OverSiblingIdxData,
        },
        traverser_core::TraverserCore,
        Traversal, Traverser,
    },
    Dyn, DynTree, NodeRef,
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

type Item<'a, O> = <O as Over>::NodeItem<'a, Dyn<i32>, Auto, SplitRecursive>;

fn dfs_iter_for<O: Over<Enumeration = Val>>()
where
    O::Enumeration: DepthFirstEnumeration,
{
    fn data<'a, O: Over>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut traverser = Dfs::<O>::new();

    let root = tree.root();
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
    fn test(mut traverser: Dfs<OverDepthData>) {
        let tree = tree();

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 1, 2, 1, 2, 2]);
    }

    test(Dfs::<OverDepthData>::new());
    test(Dfs::new());
    test(Dfs::default().with_depth());
    test(Dfs::default().with_depth().over_nodes().over_data());

    test(Traversal.dfs().with_depth());
}

#[test]
fn dfs_iter_ref_sibling() {
    fn test(mut traverser: Dfs<OverSiblingIdxData>) {
        let tree = tree();

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 0, 1, 0, 1]);
    }

    test(Dfs::<OverSiblingIdxData>::new());
    test(Dfs::new());
    test(Dfs::default().with_sibling_idx());
    test(Dfs::default().with_sibling_idx().over_nodes().over_data());

    test(Traversal.dfs().with_sibling_idx());
}

#[test]
fn dfs_iter_ref_depth_sibling() {
    fn test(mut traverser: Dfs<OverDepthSiblingIdxData>) {
        let tree = tree();

        let root = tree.root();
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

    test(Dfs::<OverDepthSiblingIdxData>::new());
    test(Dfs::new());
    test(Dfs::default().with_sibling_idx().with_depth());
    test(Dfs::default().with_depth().with_sibling_idx());
    test(
        Dfs::default()
            .with_sibling_idx()
            .with_depth()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.dfs().with_depth().with_sibling_idx());
    test(Traversal.dfs().with_sibling_idx().with_depth());
}
