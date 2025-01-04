use crate::{
    node_ref::NodeRefCore,
    traversal::{
        node_item::NodeItem,
        post_order::{iter_ref::PostOrderIterRef, PostOrderIterPtr},
        DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val,
    },
    AsTreeNode, Dyn, DynTree, Node, NodeRef,
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
    let mut iter = PostOrderIterRef::<_, _, _, Val, _, NodePtr<_>>::from((&tree.0, iter));
    assert_eq!(iter.next(), None);
}

fn post_order_iter_for<'a, D>()
where
    D: NodeItem<'a, Dyn<i32>> + 'a,
{
    fn data<'a, D>(iter: impl Iterator<Item = D>) -> Vec<<Dyn<i32> as Variant>::Item>
    where
        D: NodeItem<'a, Dyn<i32>> + 'a,
    {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, Val, _, NodePtr<_>>::from((root.col(), iter));
    assert_eq!(data(iter), [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, Val, _, NodePtr<_>>::from((root.col(), iter));
    assert_eq!(data(iter), [9, 6, 10, 11, 7, 3]);

    let n7 = n3.child(1).unwrap();
    let ptr = n7.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, Val, _, NodePtr<_>>::from((root.col(), iter));
    assert_eq!(data(iter), [10, 11, 7]);
}

#[test]
fn post_order_iter_ref_ptr() {
    post_order_iter_for::<'_, NodePtr<Dyn<i32>>>();
}

#[test]
fn post_order_iter_ref_val() {
    post_order_iter_for::<'_, Node<'_, Dyn<i32>>>();
}

#[test]
fn post_order_iter_ref_node() {
    post_order_iter_for::<'_, &i32>();
}

#[test]
fn post_order_iter_ref_depth() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, DepthVal, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]
    );

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, DepthVal, _, &i32>::from((root.col(), iter));
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [2, 1, 2, 2, 1, 0]);
}

#[test]
fn post_order_iter_ref_sibling() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, SiblingIdxVal, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.map(|x| x.0).collect::<Vec<_>>(),
        [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]
    );

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr));
    let iter = PostOrderIterRef::<_, _, _, SiblingIdxVal, _, &i32>::from((root.col(), iter));
    assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 0, 1, 1, 0]);
}

#[test]
fn post_order_iter_ref_depth_sibling() {
    let tree = tree();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr));
    let iter = PostOrderIterRef::<_, _, _, DepthSiblingIdxVal, _, &i32>::from((root.col(), iter));
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
    let iter = PostOrderIterRef::<_, _, _, DepthSiblingIdxVal, _, &i32>::from((root.col(), iter));
    assert_eq!(
        iter.clone().map(|x| x.0).collect::<Vec<_>>(),
        [2, 1, 2, 2, 1, 0]
    );

    assert_eq!(iter.map(|x| x.1).collect::<Vec<_>>(), [0, 0, 0, 1, 1, 0]);
}
