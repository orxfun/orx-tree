use crate::{
    traversal::{
        over::{OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
        post_order::traverser::PostOrder,
        traverser_mut::TraverserMut,
        Traverser,
    },
    AsTreeNode, Dyn, DynTree,
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

#[test]
fn dfs_iter_mut_val() {
    let mut tree = tree();
    let mut traverser = PostOrder::<Dyn<i32>, OverData>::default();

    let mut root = tree.root_mut().unwrap();
    let iter = traverser.iter_mut(&mut root);
    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [8, 104, 205, 302, 409, 506, 610, 711, 807, 903, 1001]
    );
}

#[test]
fn dfs_iter_mut_depth() {
    fn test(mut traverser: PostOrder<Dyn<i32>, OverDepthData>) {
        let mut tree = tree();

        let mut root = tree.root_mut().unwrap();
        let iter = traverser.iter_mut(&mut root);
        for (d, x) in iter {
            *x += 100 * d as i32;
        }

        let root = tree.root().unwrap();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| *x.1).collect::<Vec<_>>(),
            [308, 204, 205, 102, 309, 206, 310, 311, 207, 103, 1]
        );
    }

    test(PostOrder::<Dyn<i32>, OverDepthData>::default());
    test(PostOrder::default());
    test(PostOrder::<_, OverData>::default().with_depth());
    test(
        PostOrder::<_, OverData>::default()
            .with_depth()
            .over_nodes()
            .over_data(),
    );
}

#[test]
fn dfs_iter_mut_sibling() {
    fn test(mut traverser: PostOrder<Dyn<i32>, OverSiblingIdxData>) {
        let mut tree = tree();

        let mut root = tree.root_mut().unwrap();
        let iter = traverser.iter_mut(&mut root);
        for (s, x) in iter {
            *x += 100 * s as i32;
        }

        let root = tree.root().unwrap();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| *x.1).collect::<Vec<_>>(),
            [8, 4, 105, 2, 9, 6, 10, 111, 107, 103, 1]
        );
    }

    test(PostOrder::<Dyn<i32>, OverSiblingIdxData>::default());
    test(PostOrder::default());
    test(PostOrder::<_, OverData>::default().with_sibling_idx());
    test(
        PostOrder::<_, OverData>::default()
            .with_sibling_idx()
            .over_nodes()
            .over_data(),
    );
}

#[test]
fn dfs_iter_mut_depth_sibling() {
    fn test(mut traverser: PostOrder<Dyn<i32>, OverDepthSiblingIdxData>) {
        let mut tree = tree();

        let mut root = tree.root_mut().unwrap();
        let iter = traverser.iter_mut(&mut root);
        for (d, s, x) in iter {
            *x += 10000 * d as i32 + 100 * s as i32;
        }

        let root = tree.root().unwrap();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| *x.2).collect::<Vec<_>>(),
            [30008, 20004, 20105, 10002, 30009, 20006, 30010, 30111, 20107, 10103, 1]
        );
    }

    test(PostOrder::<Dyn<i32>, OverDepthSiblingIdxData>::default());
    test(PostOrder::default());
    test(
        PostOrder::<_, OverData>::default()
            .with_sibling_idx()
            .with_depth(),
    );
    test(
        PostOrder::<_, OverData>::default()
            .with_depth()
            .with_sibling_idx(),
    );
    test(
        PostOrder::<_, OverData>::default()
            .with_sibling_idx()
            .with_depth()
            .over_nodes()
            .over_data(),
    );
}
