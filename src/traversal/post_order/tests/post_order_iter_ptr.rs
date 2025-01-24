use crate::{
    node_ref::NodeRefCore,
    traversal::{enumerations::Val, post_order::iter_ptr::PostOrderIterPtr},
    Dyn, DynTree, NodeRef, TreeVariant,
};
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

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

    let mut n2 = tree.node_mut(&id2);
    let [id4, _] = n2.push_children([4, 5]);

    tree.node_mut(&id4).push_child(8);

    let mut n3 = tree.node_mut(&id3);
    let [id6, id7] = n3.push_children([6, 7]);

    tree.node_mut(&id6).push_child(9);
    tree.node_mut(&id7).push_children([10, 11]);

    tree
}

fn data<V: TreeVariant>(iter_ptr: impl Iterator<Item = NodePtr<V>>) -> Vec<V::Item>
where
    V::Item: Clone,
{
    iter_ptr
        .map(|x| {
            let node = unsafe { &*x.ptr() };
            node.data().unwrap().clone()
        })
        .collect()
}

#[test]
fn post_order_iter_ptr_empty() {
    let mut iter = PostOrderIterPtr::<Dyn<i32>, Val>::default();
    assert_eq!(iter.next(), None);
}

#[test]
fn post_order_iter_ptr() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root();
    let ptr = root.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    assert_eq!(data(iter), [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr));
    assert_eq!(data(iter), [9, 6, 10, 11, 7, 3]);

    let n7 = n3.child(1).unwrap();
    let ptr = n7.node_ptr().clone();
    let iter = PostOrderIterPtr::<_, Val, _>::from((stack, ptr));
    assert_eq!(data(iter), [10, 11, 7]);
}
