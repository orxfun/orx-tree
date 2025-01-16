use crate::{
    traversal::{
        over::{OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
        post_order::traverser::PostOrder,
        traverser_core::TraverserCore,
        Traversal, Traverser,
    },
    DynTree,
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

#[test]
fn post_order_iter_mut_val() {
    let mut tree = tree();
    let mut traverser = PostOrder::<OverData>::default();

    let mut root = tree.root_mut();
    let iter = traverser.iter_mut(&mut root);
    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [8, 104, 205, 302, 409, 506, 610, 711, 807, 903, 1001]
    );
}

#[test]
fn post_order_iter_mut_depth() {
    fn test(mut traverser: PostOrder<OverDepthData>) {
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
            [308, 204, 205, 102, 309, 206, 310, 311, 207, 103, 1]
        );
    }

    test(PostOrder::<OverDepthData>::new());
    test(PostOrder::new());
    test(PostOrder::default().with_depth());
    test(PostOrder::default().with_depth().over_nodes().over_data());

    test(Traversal.post_order().with_depth());
}

#[test]
fn post_order_iter_mut_sibling() {
    fn test(mut traverser: PostOrder<OverSiblingIdxData>) {
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
            [8, 4, 105, 2, 9, 6, 10, 111, 107, 103, 1]
        );
    }

    test(PostOrder::<OverSiblingIdxData>::new());
    test(PostOrder::new());
    test(PostOrder::default().with_sibling_idx());
    test(
        PostOrder::default()
            .with_sibling_idx()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.post_order().with_sibling_idx());
}

#[test]
fn post_order_iter_mut_depth_sibling() {
    fn test(mut traverser: PostOrder<OverDepthSiblingIdxData>) {
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
            [30008, 20004, 20105, 10002, 30009, 20006, 30010, 30111, 20107, 10103, 1]
        );
    }

    test(PostOrder::<OverDepthSiblingIdxData>::new());
    test(PostOrder::new());
    test(PostOrder::default().with_sibling_idx().with_depth());
    test(PostOrder::default().with_depth().with_sibling_idx());
    test(
        PostOrder::default()
            .with_sibling_idx()
            .with_depth()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.post_order().with_depth().with_sibling_idx());
    test(Traversal.post_order().with_sibling_idx().with_depth());
}
