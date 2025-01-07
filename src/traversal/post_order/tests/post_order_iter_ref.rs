use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
        node_item::NodeItem,
        over::{Over, OverData, OverNode, OverPtr},
        post_order::{iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef},
    },
    AsTreeNode, Dyn, DynTree, NodeRef,
};
use alloc::vec::Vec;
use orx_selfref_col::{NodePtr, Variant};

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
fn post_order_iter_ref_empty() {
    let tree = DynTree::<i32>::empty();
    let iter = PostOrderIterPtr::<Dyn<i32>, Val>::default();
    let mut iter = PostOrderIterRef::<_, Auto, SplitRecursive, Val, _, NodePtr<_>>::from(
        (&tree.0, iter),
    );
    assert_eq!(iter.next(), None);
}

type Item<'a, O> = <O as Over<Dyn<i32>>>::NodeItem<'a, Auto, SplitRecursive>;

fn post_order_iter_for<O: Over<Dyn<i32>>>() {
    fn data<'a, O: Over<Dyn<i32>> + 'a>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, Val, _, Item<'_, O>>::from((root.col(), iter));
    assert_eq!(data::<'_, O>(iter), [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, Val, _, Item<'_, O>>::from((root.col(), iter));
    assert_eq!(data::<'_, O>(iter), [9, 6, 10, 11, 7, 3]);

    let n7 = n3.child(1).unwrap();
    let ptr = n7.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, Val, _, Item<'_, O>>::from((root.col(), iter));
    assert_eq!(data::<'_, O>(iter), [10, 11, 7]);
}

#[test]
fn post_order_iter_ref_ptr() {
    post_order_iter_for::<OverPtr>();
}

#[test]
fn post_order_iter_ref_val() {
    post_order_iter_for::<OverNode>();
}

#[test]
fn post_order_iter_ref_node() {
    post_order_iter_for::<OverData>();
}

#[test]
fn post_order_iter_ref_depth() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, Auto, SplitRecursive, DepthVal, _, &i32>::from((
        root.col(),
        iter,
    ));
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]
    );

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, Auto, SplitRecursive, DepthVal, _, &i32>::from((
        root.col(),
        iter,
    ));
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [2, 1, 2, 2, 1, 0]);
}

#[test]
fn post_order_iter_ref_sibling() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, Auto, SplitRecursive, SiblingIdxVal, _, &i32>::from(
        (root.col(), iter),
    );
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]
    );

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, Auto, SplitRecursive, SiblingIdxVal, _, &i32>::from(
        (root.col(), iter),
    );
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 0, 1, 1, 0]);
}

#[test]
fn post_order_iter_ref_depth_sibling() {
    let tree = tree();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr));
    let iter =
        PostOrderIterRef::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _, &i32>::from((
            root.col(),
            iter,
        ));
    assert_eq!(
        iter.clone().map(|x| x.0).collect::<Vec<_>>(),
        [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]
    );

    assert_eq!(
        iter.map(|x| x.1).collect::<Vec<_>>(),
        [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]
    );

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr));
    let iter =
        PostOrderIterRef::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _, &i32>::from((
            root.col(),
            iter,
        ));
    assert_eq!(
        iter.clone().map(|x| x.0).collect::<Vec<_>>(),
        [2, 1, 2, 2, 1, 0]
    );

    assert_eq!(iter.map(|x| x.1).collect::<Vec<_>>(), [0, 0, 0, 1, 1, 0]);
}
