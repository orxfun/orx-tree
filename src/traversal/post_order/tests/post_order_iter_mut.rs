use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    traversal::{
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
        post_order::{
            iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
        },
    },
    AsTreeNode, Dyn, DynTree,
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
fn post_order_iter_ref_empty() {
    let mut tree = DynTree::<i32>::empty();
    let iter = PostOrderIterPtr::<Dyn<i32>, Val>::default();
    let mut iter =
        unsafe { PostOrderIterMut::<_, Auto, _, Val, _, &mut i32>::from((&mut tree.0, iter)) };
    assert_eq!(iter.next(), None);
}

#[test]
fn post_order_iter_mut_val() {
    let mut tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter =
        unsafe { PostOrderIterMut::<_, Auto, _, Val, _, &mut i32>::from((&mut tree.0, iter)) };

    for (i, x) in iter.enumerate() {
        *x += 100 * i as i32;
    }

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, Auto, _, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [8, 104, 205, 302, 409, 506, 610, 711, 807, 903, 1001]
    );
}

#[test]
fn post_order_iter_mut_depth() {
    let mut tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter =
        unsafe { PostOrderIterMut::<_, Auto, _, DepthVal, _, &mut i32>::from((&mut tree.0, iter)) };

    for (d, x) in iter {
        *x += 100 * d as i32;
    }

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::new(), ptr));
    let iter = PostOrderIterRef::<_, Auto, _, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [308, 204, 205, 102, 309, 206, 310, 311, 207, 103, 1]
    );
}

#[test]
fn post_order_iter_mut_sibling() {
    let mut tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = unsafe {
        PostOrderIterMut::<_, Auto, _, SiblingIdxVal, _, &mut i32>::from((&mut tree.0, iter))
    };

    for (s, x) in iter {
        *x += 100 * s as i32;
    }

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::new(), ptr));
    let iter = PostOrderIterRef::<_, Auto, _, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [8, 4, 105, 2, 9, 6, 10, 111, 107, 103, 1]
    );
}

#[test]
fn post_order_iter_mut_depth_sibling() {
    let mut tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = unsafe {
        PostOrderIterMut::<_, Auto, _, DepthSiblingIdxVal, _, &mut i32>::from((&mut tree.0, iter))
    };

    for (d, s, x) in iter {
        *x += 10000 * d as i32 + 100 * s as i32;
    }

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::new(), ptr));
    let iter = PostOrderIterRef::<_, Auto, _, Val, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.copied().collect::<Vec<_>>(),
        [30008, 20004, 20105, 10002, 30009, 20006, 30010, 30111, 20107, 10103, 1]
    );
}
