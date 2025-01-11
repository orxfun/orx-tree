use crate::{
    memory::Auto,
    pinned_storage::SplitRecursive,
    traversal::{
        enumerations::Val,
        node_item::NodeItem,
        over::{
            Over, OverData, OverDepthData, OverDepthSiblingIdxData, OverNode, OverPtr,
            OverSiblingIdxData,
        },
        post_order::{post_enumeration::PostOrderEnumeration, traverser::PostOrder},
        traverser_core::TraverserCore,
        Traversal, Traverser,
    },
    Dyn, DynTree, NodeRef,
};
use alloc::vec::Vec;
use orx_selfref_col::Variant;

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

type Item<'a, O> = <O as Over>::NodeItem<'a, Dyn<i32>, Auto, SplitRecursive>;

fn post_order_iter_for<O: Over<Enumeration = Val>>()
where
    O::Enumeration: PostOrderEnumeration,
{
    fn data<'a, O: Over>(
        iter: impl Iterator<Item = Item<'a, O>>,
    ) -> Vec<<Dyn<i32> as Variant>::Item> {
        iter.map(|x| x.node_data().clone()).collect()
    }

    let tree = tree();
    let mut traverser = PostOrder::<O>::new();

    let root = tree.root();
    let iter = traverser.iter(&root);
    assert_eq!(data::<O>(iter), [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);

    let n3 = root.child(1).unwrap();
    let iter = traverser.iter(&n3);
    assert_eq!(data::<O>(iter), [9, 6, 10, 11, 7, 3]);

    let n7 = n3.child(1).unwrap();
    let iter = traverser.iter(&n7);
    assert_eq!(data::<O>(iter), [10, 11, 7]);
}

#[test]
fn post_order_traverser_ptr() {
    post_order_iter_for::<OverPtr>();
}

#[test]
fn post_order_traverser_val() {
    post_order_iter_for::<OverNode>();
}

#[test]
fn post_order_traverser_node() {
    post_order_iter_for::<OverData>();
}

#[test]
fn post_order_iter_ref_depth() {
    fn test(mut traverser: PostOrder<OverDepthData>) {
        let tree = tree();

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [2, 1, 2, 2, 1, 0]);
    }

    test(PostOrder::<OverDepthData>::new());
    test(PostOrder::new());
    test(PostOrder::default().with_depth());
    test(PostOrder::default().with_depth().over_nodes().over_data());

    test(Traversal.post_order().with_depth());
}

#[test]
fn post_order_iter_ref_sibling() {
    fn test(mut traverser: PostOrder<OverSiblingIdxData>) {
        let tree = tree();

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [0, 0, 0, 1, 1, 0]);
    }

    test(PostOrder::<OverSiblingIdxData>::new());
    test(PostOrder::new());
    test(PostOrder::default().with_sibling_idx());
    test(
        PostOrder::default()
            .with_sibling_idx()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.post_order().with_sibling_idx());
}

#[test]
fn post_order_iter_ref_depth_sibling() {
    fn test(mut traverser: PostOrder<OverDepthSiblingIdxData>) {
        let tree = tree();

        let root = tree.root();
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.0).collect::<Vec<_>>(),
            [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]
        );
        let iter = traverser.iter(&root);
        assert_eq!(
            iter.map(|x| x.1).collect::<Vec<_>>(),
            [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]
        );

        let n3 = root.child(1).unwrap();
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.0).collect::<Vec<_>>(), [2, 1, 2, 2, 1, 0]);
        let iter = traverser.iter(&n3);
        assert_eq!(iter.map(|x| x.1).collect::<Vec<_>>(), [0, 0, 0, 1, 1, 0]);
    }

    test(PostOrder::<OverDepthSiblingIdxData>::new());
    test(PostOrder::new());
    test(PostOrder::default().with_sibling_idx().with_depth());
    test(PostOrder::default().with_depth().with_sibling_idx());
    test(
        PostOrder::default()
            .with_sibling_idx()
            .with_depth()
            .over_nodes()
            .over_data(),
    );

    test(Traversal.post_order().with_depth().with_sibling_idx());
    test(Traversal.post_order().with_sibling_idx().with_depth());
}
