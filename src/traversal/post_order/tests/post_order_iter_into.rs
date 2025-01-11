use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
        post_order::{into_iter::PostOrderIterInto, iter_ptr::PostOrderIterPtr},
    },
    Bfs, DynTree, NodeRef,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use test_case::test_matrix;

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
fn tree() -> DynTree<String> {
    let mut tree = DynTree::<String>::new(1.to_string());

    let mut root = tree.root_mut();
    let [id2, id3] = root.grow([2.to_string(), 3.to_string()]);

    let mut n2 = tree.node_mut(&id2);
    let [id4, _] = n2.grow([4.to_string(), 5.to_string()]);

    tree.node_mut(&id4).push(8.to_string());

    let mut n3 = tree.node_mut(&id3);
    let [id6, id7] = n3.grow([6.to_string(), 7.to_string()]);

    tree.node_mut(&id6).push(9.to_string());
    tree.node_mut(&id7).extend([10.to_string(), 11.to_string()]);

    tree
}

enum UseIter {
    Nonce,
    Once,
    All,
}

#[test_matrix([UseIter::Nonce, UseIter::Once, UseIter::All])]
fn post_order_into_iter_partially_used(use_iter: UseIter) {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr.clone()));
        {
            let mut iter = unsafe {
                PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
            };
            match use_iter {
                UseIter::All => for _ in iter {},
                UseIter::Once => {
                    let first = iter.next();
                    assert!(first.is_some());
                }
                UseIter::Nonce => {}
            }
        }

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::default(), ptr.clone()));
        {
            let mut iter = unsafe {
                PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
            };
            match use_iter {
                UseIter::All => for _ in iter {},
                UseIter::Once => {
                    let first = iter.next();
                    assert!(first.is_some());
                }
                UseIter::Nonce => {}
            }
        }

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.child(1).unwrap();
        let n7 = n3.child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::default(), ptr.clone()));
        {
            let mut iter = unsafe {
                PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
            };
            match use_iter {
                UseIter::All => for _ in iter {},
                UseIter::Once => {
                    let first = iter.next();
                    assert!(first.is_some());
                }
                UseIter::Nonce => {}
            }
        }

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn post_order_into_iter_val() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(
            values,
            [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1].map(|x| x.to_string())
        );

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3].map(|x| x.to_string()));

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.child(1).unwrap();
        let n7 = n3.child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = PostOrderIterPtr::<_, Val, _>::from((Vec::default(), ptr.clone()));
        let iter = unsafe {
            PostOrderIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [10, 11, 7].map(|x| x.to_string()));

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn post_order_into_iter_depth() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(
            values,
            [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1].map(|x| x.to_string())
        );
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [2, 1, 2, 2, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [10, 11, 7].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [1, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn post_order_into_iter_sibling_idx() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(
            values,
            [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1].map(|x| x.to_string())
        );
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3].map(|x| x.to_string()));
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 0, 1, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [10, 11, 7].map(|x| x.to_string()));
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn post_order_into_iter_depth_sibling_idx() {
    {
        let mut tree = tree();
        let mut stack = Vec::default();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.2.clone()).collect();
        assert_eq!(
            values,
            [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1].map(|x| x.to_string())
        );
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [3, 2, 2, 1, 3, 2, 3, 3, 2, 1, 0]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.2.clone()).collect();
        assert_eq!(values, [9, 6, 10, 11, 7, 3].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [2, 1, 2, 2, 1, 0]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 0, 1, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
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
        let values: Vec<_> = result.iter().map(|x| x.2.clone()).collect();
        assert_eq!(values, [10, 11, 7].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [1, 1, 0]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 1, 0]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.get_root().unwrap().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}
