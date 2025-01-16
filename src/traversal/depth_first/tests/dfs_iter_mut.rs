use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        depth_first::{iter_mut::DfsIterMut, iter_ptr::DfsIterPtr, iter_ref::DfsIterRef},
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
    },
    Dyn, DynTree,
};
use alloc::vec::Vec;

/// ```
///       1
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
fn dfs_iter_ref_empty() {
    let tree = DynTree::<i32>::empty();
    let iter = DfsIterPtr::<Dyn<i32>, Val>::default();
    let mut iter =
        unsafe { DfsIterMut::<_, Auto, SplitRecursive, Val, _, &mut i32>::from((&tree.0, iter)) };
    assert_eq!(iter.next(), None);
}

#[test]
fn dfs_iter_mut_val() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter =
        unsafe { DfsIterMut::<_, Auto, SplitRecursive, Val, _, &mut i32>::from((&tree.0, iter)) };

    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 102, 204, 308, 405, 503, 606, 709, 807, 910, 1011]
    );
}

#[test]
fn dfs_iter_mut_depth() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = unsafe {
        DfsIterMut::<_, Auto, SplitRecursive, DepthVal, _, &mut i32>::from((&tree.0, iter))
    };

    for (d, x) in iter {
        *x += 100 * d as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((Vec::new(), ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 102, 204, 308, 205, 103, 206, 309, 207, 310, 311]
    );
}

#[test]
fn dfs_iter_mut_sibling() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = unsafe {
        DfsIterMut::<_, Auto, SplitRecursive, SiblingIdxVal, _, &mut i32>::from((&tree.0, iter))
    };

    for (s, x) in iter {
        *x += 100 * s as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((Vec::new(), ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 2, 4, 8, 105, 103, 6, 9, 107, 10, 111]
    );
}

#[test]
fn dfs_iter_mut_depth_sibling() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, DepthSiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = unsafe {
        DfsIterMut::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _, &mut i32>::from((
            &tree.0, iter,
        ))
    };

    for (d, s, x) in iter {
        *x += 10000 * d as i32 + 100 * s as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((Vec::new(), ptr));
    let iter = DfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 10002, 20004, 30008, 20105, 10103, 20006, 30009, 20107, 30010, 30111]
    );
}
