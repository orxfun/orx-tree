use crate::{
    node_ref::NodeRefCore,
    traversal::{depth_first::iter_ptr::DfsIterPtr, enumerations::Val},
    Dyn, DynTree, NodeRef, TreeVariant,
};
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

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
fn dfs_iter_ptr_empty() {
    let mut iter = DfsIterPtr::<Dyn<i32>, Val>::default();
    assert_eq!(iter.next(), None);
}

#[test]
fn dfs_iter_ptr() {
    let tree = tree();
    let mut stack = Vec::default();

    let root = tree.root().unwrap();
    let ptr = root.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    assert_eq!(data(iter), [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    let n3 = root.child(1).unwrap();
    let ptr = n3.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((&mut stack, ptr));
    assert_eq!(data(iter), [3, 6, 9, 7, 10, 11]);

    let n7 = n3.child(1).unwrap();
    let ptr = n7.node_ptr().clone();
    let iter = DfsIterPtr::<_, Val, _>::from((stack, ptr));
    assert_eq!(data(iter), [7, 10, 11]);
}
