use crate::{
    memory::Auto,
    pinned_storage::SplitRecursive,
    traversal::{
        breadth_first::traverser::Bfs,
        enumerations::Val,
        node_item::NodeItem,
        over::{
            Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverNode, OverPtr,
            OverSiblingIdxData,
        },
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

type Item<'a, O> = <O as Over>::NodeItem<'a, Dyn<i32>, Auto, SplitRecursive>;

fn bfs_iter_for<O: Over<Enumeration = Val>>() {
    fn data<'a, O: Over + 'a>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut traverser = Bfs::<Dyn<i32>, O>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(data::<O>(iter), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(data::<O>(iter), [3, 6, 7, 9, 10, 11]);

    let n7 = n3.child(1).unwrap();
    let iter = traverser.iter(&n7);
    assert_eq!(data::<O>(iter), [7, 10, 11]);
}

#[test]
fn bfs_traverser_ptr() {
    bfs_iter_for::<OverPtr>();
}

#[test]
fn bfs_traverser_val() {
    bfs_iter_for::<OverNode>();
}

#[test]
fn bfs_traverser_node() {
    bfs_iter_for::<OverData>();
}

#[test]
fn bfs_iter_ref_depth() {
    fn test(mut traverser: Bfs<Dyn<i32>, OverDepthData>) {
        let tree = tree();

        let root = tree.root().unwrap();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 1, 1, 2, 2, 2]);
    }

    test(Bfs::<Dyn<i32>, OverDepthData>::default());
    test(Bfs::default());
    test(Bfs::<_, OverData>::default().with_depth());
    test(
        Bfs::<_, OverData>::default()
            .with_depth()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.bfs().with_depth());
}

#[test]
fn bfs_iter_ref_sibling() {
    fn test(mut traverser: Bfs<Dyn<i32>, OverSiblingIdxData>) {
        let tree = tree();

        let root = tree.root().unwrap();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 1, 0, 0, 1]);
    }

    test(Bfs::<Dyn<i32>, OverSiblingIdxData>::default());
    test(Bfs::default());
    test(Bfs::<_, OverData>::default().with_sibling_idx());
    test(
        Bfs::<_, OverData>::default()
            .with_sibling_idx()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.bfs().with_sibling_idx());
}

#[test]
fn bfs_iter_ref_depth_sibling() {
    fn test(mut traverser: Bfs<Dyn<i32>, OverDepthSiblingIdxData>) {
        let tree = tree();

        let root = tree.root().unwrap();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]
        );
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.1).collect::<Vec<_>>(),
            [0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 1, 1, 2, 2, 2]);
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.1).collect::<Vec<_>>(), [0, 0, 1, 0, 0, 1]);
    }

    test(Bfs::<Dyn<i32>, OverDepthSiblingIdxData>::default());
    test(Bfs::default());
    test(
        Bfs::<_, OverData>::default()
            .with_sibling_idx()
            .with_depth(),
    );
    test(
        Bfs::<_, OverData>::default()
            .with_depth()
            .with_sibling_idx(),
    );
    test(
        Bfs::<_, OverData>::default()
            .with_sibling_idx()
            .with_depth()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.bfs().with_depth().with_sibling_idx());
    test(Traversal.bfs().with_sibling_idx().with_depth());
}
