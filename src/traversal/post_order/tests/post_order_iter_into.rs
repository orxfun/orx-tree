use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
        over::Over,
        post_order::{
            into_iter::PostOrderIterInto, iter_ptr::PostOrderIterPtr, iter_ref::PostOrderIterRef,
        },
    },
    Dyn, DynTree, NodeRef,
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
    let mut iter =
        PostOrderIterRef::<_, Auto, SplitRecursive, Val, _, NodePtr<_>>::from((&tree.0, iter));
    assert_eq!(iter.next(), None);
}

#[test]
fn post_order_into_iter_val() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root().unwrap();
        let ptr = root.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 4, 5, 8]);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let n7 = n3.child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [10, 11, 7]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9]);
    }
}

#[test]
fn post_order_into_iter_depth() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root().unwrap();
        let ptr = root.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, DepthVal, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, DepthVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, DepthVal, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, DepthVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3]);
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [2, 1, 2, 2, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 4, 5, 8]);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let n7 = n3.child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, DepthVal, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, DepthVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(values, [10, 11, 7]);
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [1, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9]);
    }
}

#[test]
fn post_order_into_iter_sibling_idx() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root().unwrap();
        let ptr = root.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, SiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, SiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3]);
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 0, 1, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 4, 5, 8]);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let n7 = n3.child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, SiblingIdxVal, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, SiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(values, [10, 11, 7]);
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9]);
    }
}

#[test]
fn post_order_into_iter_depth_sibling_idx() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root().unwrap();
        let ptr = root.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.2).collect();
        assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter =
            PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.2).collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3]);
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [2, 1, 2, 2, 1, 0]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 0, 1, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 4, 5, 8]);
    }

    {
        let mut tree = tree();

        let root = tree.root().unwrap();
        let n3 = root.child(1).unwrap();
        let n7 = n3.child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter =
            PostOrderIterPtr::<_, DepthSiblingIdxVal, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.2).collect();
        assert_eq!(values, [10, 11, 7]);
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [1, 1, 0]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(tree.root().map(|x| *x.data()), Some(1));
        let values: Vec<_> = tree.root().unwrap().bfs().copied().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9]);
    }
}
