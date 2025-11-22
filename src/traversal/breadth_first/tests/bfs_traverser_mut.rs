use crate::{
    DynTree,
    traversal::{
        Traversal, Traverser,
        breadth_first::traverser::Bfs,
        over::{OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
        traverser_core::TraverserCore,
    },
};
use alloc::vec::Vec;

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

    let mut n2 = tree.node_mut(id2);
    let [id4, _] = n2.push_children([4, 5]);

    tree.node_mut(id4).push_child(8);

    let mut n3 = tree.node_mut(id3);
    let [id6, id7] = n3.push_children([6, 7]);

    tree.node_mut(id6).push_child(9);
    tree.node_mut(id7).push_children([10, 11]);

    tree
}

#[test]
fn bfs_iter_mut_val() {
    let mut tree = tree();
    let mut traverser = Bfs::<OverData>::default();

    let mut root = tree.root_mut();
    let iter = traverser.iter_mut(&mut root);
    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 102, 203, 304, 405, 506, 607, 708, 809, 910, 1011]
    );
}

#[test]
fn bfs_iter_mut_depth() {
    fn test(mut traverser: Bfs<OverDepthData>) {
        let mut tree = tree();

        let mut root = tree.root_mut();
        let iter = traverser.iter_mut(&mut root);
        for (d, x) in iter {
            *x += 100 * d as i32;
        }

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| *x.1).collect::<Vec<_>>(),
            [1, 102, 103, 204, 205, 206, 207, 308, 309, 310, 311]
        );
    }

    test(Bfs::<OverDepthData>::new());
    test(Bfs::default().with_depth());
    test(Bfs::default().with_depth().over_nodes().over_data());

    test(Traversal.bfs().with_depth());
}

#[test]
fn bfs_iter_mut_sibling() {
    fn test(mut traverser: Bfs<OverSiblingIdxData>) {
        let mut tree = tree();

        let mut root = tree.root_mut();
        let iter = traverser.iter_mut(&mut root);
        for (s, x) in iter {
            *x += 100 * s as i32;
        }

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| *x.1).collect::<Vec<_>>(),
            [1, 2, 103, 4, 105, 6, 107, 8, 9, 10, 111]
        );
    }

    test(Bfs::<OverSiblingIdxData>::new());
    test(Bfs::new());
    test(Bfs::default().with_sibling_idx());
    test(Bfs::default().with_sibling_idx().over_nodes().over_data());

    test(Traversal.bfs().with_sibling_idx());
}

#[test]
fn bfs_iter_mut_depth_sibling() {
    fn test(mut traverser: Bfs<OverDepthSiblingIdxData>) {
        let mut tree = tree();

        let mut root = tree.root_mut();
        let iter = traverser.iter_mut(&mut root);
        for (d, s, x) in iter {
            *x += 10000 * d as i32 + 100 * s as i32;
        }

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| *x.2).collect::<Vec<_>>(),
            [
                1, 10002, 10103, 20004, 20105, 20006, 20107, 30008, 30009, 30010, 30111
            ]
        );
    }

    test(Bfs::<OverDepthSiblingIdxData>::new());
    test(Bfs::new());
    test(Bfs::default().with_sibling_idx().with_depth());
    test(Bfs::default().with_depth().with_sibling_idx());
    test(
        Bfs::default()
            .with_sibling_idx()
            .with_depth()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.bfs().with_depth().with_sibling_idx());
    test(Traversal.bfs().with_sibling_idx().with_depth());
}
