use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        breadth_first::{iter_mut::BfsIterMut, iter_ptr::BfsIterPtr, iter_ref::BfsIterRef},
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
    },
    Dyn, DynTree,
};
use alloc::collections::VecDeque;
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
    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = tree.node_mut(&id2);
    let [id4, _] = n2.grow([4, 5]);

    tree.node_mut(&id4).push(8);

    let mut n3 = tree.node_mut(&id3);
    let [id6, id7] = n3.grow([6, 7]);

    tree.node_mut(&id6).push(9);
    tree.node_mut(&id7).extend([10, 11]);

    tree
}

#[test]
fn bfs_iter_ref_empty() {
    let tree = DynTree::<i32>::empty();
    let iter = BfsIterPtr::<Dyn<i32>, Val>::default();
    let mut iter =
        unsafe { BfsIterMut::<_, Auto, SplitRecursive, Val, _, &mut i32>::from((&tree.0, iter)) };
    assert_eq!(iter.next(), None);
}

#[test]
fn bfs_iter_mut_val() {
    let tree = tree();
    let mut queue = VecDeque::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, Val, _>::from((&mut queue, ptr));
    let iter =
        unsafe { BfsIterMut::<_, Auto, SplitRecursive, Val, _, &mut i32>::from((&tree.0, iter)) };

    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, Val, _>::from((&mut queue, ptr));
    let iter = BfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 102, 203, 304, 405, 506, 607, 708, 809, 910, 1011]
    );
}

#[test]
fn bfs_iter_mut_depth() {
    let tree = tree();
    let mut queue = VecDeque::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, DepthVal, _>::from((&mut queue, ptr));
    let iter = unsafe {
        BfsIterMut::<_, Auto, SplitRecursive, DepthVal, _, &mut i32>::from((&tree.0, iter))
    };

    for (d, x) in iter {
        *x += 100 * d as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::new(), ptr));
    let iter = BfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 102, 103, 204, 205, 206, 207, 308, 309, 310, 311]
    );
}

#[test]
fn bfs_iter_mut_sibling() {
    let tree = tree();
    let mut queue = VecDeque::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, SiblingIdxVal, _>::from((&mut queue, ptr));
    let iter = unsafe {
        BfsIterMut::<_, Auto, SplitRecursive, SiblingIdxVal, _, &mut i32>::from((&tree.0, iter))
    };

    for (s, x) in iter {
        *x += 100 * s as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::new(), ptr));
    let iter = BfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 2, 103, 4, 105, 6, 107, 8, 9, 10, 111]
    );
}

#[test]
fn bfs_iter_mut_depth_sibling() {
    let tree = tree();
    let mut queue = VecDeque::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, DepthSiblingIdxVal, _>::from((&mut queue, ptr));
    let iter = unsafe {
        BfsIterMut::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _, &mut i32>::from((
            &tree.0, iter,
        ))
    };

    for (d, s, x) in iter {
        *x += 10000 * d as i32 + 100 * s as i32;
    }

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::new(), ptr));
    let iter = BfsIterRef::<_, Auto, SplitRecursive, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [1, 10002, 10103, 20004, 20105, 20006, 20107, 30008, 30009, 30010, 30111]
    );
}
