use crate::{
    node_ref::NodeRefCore,
    traversal::{
        depth_first::{
            dfs::Dfs, dfs_enumeration::DepthFirstEnumeration, iter_ref::DfsIterRef, DfsIterPtr,
        },
        node_item::NodeItem,
        over::{Over, OverData, OverNode, OverPtr},
        DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Traverser, Val,
    },
    tree::{DefaultMemory, DefaultPinVec},
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

type Item<'a, O> =
    <O as Over<Dyn<i32>>>::NodeItem<'a, DefaultMemory<Dyn<i32>>, DefaultPinVec<Dyn<i32>>>;

fn dfs_iter_for<O: Over<Dyn<i32>, Enumeration = Val>>()
where
    O::Enumeration: DepthFirstEnumeration,
{
    fn data<'a, O: Over<Dyn<i32>> + 'a>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut traverser = Dfs::<Dyn<i32>, O>::default();

    let root = tree.root().unwrap();
    let iter = traverser.iter(&root);
    assert_eq!(data::<O>(iter), [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(data::<O>(iter), [3, 6, 9, 7, 10, 11]);

    let n7 = n3.child(1).unwrap();
    let iter = traverser.iter(&n7);
    assert_eq!(data::<O>(iter), [7, 10, 11]);
}

#[test]
fn dfs_traverser_ptr() {
    dfs_iter_for::<OverPtr>();
}

#[test]
fn dfs_traverser_val() {
    dfs_iter_for::<OverNode>();
}

#[test]
fn dfs_traverser_node() {
    dfs_iter_for::<OverData>();
}
