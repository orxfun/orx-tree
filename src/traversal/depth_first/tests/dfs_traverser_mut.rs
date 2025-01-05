use crate::{
    traversal::{
        depth_first::dfs::Dfs,
        over::{OverData, OverDepthData, OverDepthSiblingIdxData, OverSiblingIdxData},
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
    let mut traverser = Dfs::<Dyn<i32>, OverData>::default();

    let mut root = tree.root_mut().unwrap();
    let iter = traverser.iter_mut(&mut root);
    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 102, 204, 308, 405, 503, 606, 709, 807, 910, 1011]
    );
}

#[test]
fn dfs_iter_mut_depth() {
    let mut tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverDepthData>::default();

    let mut root = tree.root_mut().unwrap();
    let iter = traverser.iter_mut(&mut root);
    for (d, x) in iter {
        *x += 100 * d as i32;
    }

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| *x.1).collect::<Vec<_>>(),
        [1, 102, 204, 308, 205, 103, 206, 309, 207, 310, 311]
    );
}

#[test]
fn dfs_iter_mut_sibling() {
    let mut tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverSiblingIdxData>::default();

    let mut root = tree.root_mut().unwrap();
    let iter = traverser.iter_mut(&mut root);
    for (s, x) in iter {
        *x += 100 * s as i32;
    }

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| *x.1).collect::<Vec<_>>(),
        [1, 2, 4, 8, 105, 103, 6, 9, 107, 10, 111]
    );
}

#[test]
fn dfs_iter_mut_depth_sibling() {
    let mut tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, OverDepthSiblingIdxData>::default();

    let mut root = tree.root_mut().unwrap();
    let iter = traverser.iter_mut(&mut root);
    for (d, s, x) in iter {
        *x += 10000 * d as i32 + 100 * s as i32;
    }

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(
        iter.map(|x| *x.2).collect::<Vec<_>>(),
        [1, 10002, 20004, 30008, 20105, 10103, 20006, 30009, 20107, 30010, 30111]
    );
}
