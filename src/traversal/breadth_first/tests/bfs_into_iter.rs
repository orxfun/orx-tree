use crate::{
    memory::Auto,
    node_ref::NodeRefCore,
    pinned_storage::SplitRecursive,
    traversal::{
        breadth_first::{into_iter::BfsIterInto, iter_ptr::BfsIterPtr},
        enumerations::{DepthSiblingIdxVal, DepthVal, SiblingIdxVal, Val},
    },
    Bfs, DynTree, NodeRef,
};
use alloc::collections::VecDeque;
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
    let [id2, id3] = root.push_children([2.to_string(), 3.to_string()]);

    let mut n2 = tree.node_mut(&id2);
    let [id4, _] = n2.push_children([4.to_string(), 5.to_string()]);

    tree.node_mut(&id4).push_child(8.to_string());

    let mut n3 = tree.node_mut(&id3);
    let [id6, id7] = n3.push_children([6.to_string(), 7.to_string()]);

    tree.node_mut(&id6).push_child(9.to_string());
    tree.node_mut(&id7)
        .push_children([10.to_string(), 11.to_string()]);

    tree
}

enum UseIter {
    Nonce,
    Once,
    All,
}

#[test_matrix([UseIter::Nonce, UseIter::Once, UseIter::All])]
fn bfs_into_iter_partially_used(use_iter: UseIter) {
    {
        let mut tree = tree();
        let mut stack = VecDeque::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val, _>::from((&mut stack, ptr.clone()));
        {
            let mut iter = unsafe {
                BfsIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
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
        let n3 = root.get_child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::default(), ptr.clone()));
        {
            let mut iter = unsafe {
                BfsIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
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
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let n7 = n3.get_child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::default(), ptr.clone()));
        {
            let mut iter = unsafe {
                BfsIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
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
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn bfs_into_iter_val() {
    {
        let mut tree = tree();
        let mut stack = VecDeque::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(
            values,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(|x| x.to_string())
        );

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [3, 6, 7, 9, 10, 11].map(|x| x.to_string()));

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let n7 = n3.get_child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = BfsIterPtr::<_, Val, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, Val, _>::from((&mut tree.0, iter, ptr))
        };
        let values: Vec<_> = iter.collect();
        assert_eq!(values, [7, 10, 11].map(|x| x.to_string()));

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn bfs_into_iter_depth() {
    {
        let mut tree = tree();
        let mut stack = VecDeque::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = BfsIterPtr::<_, DepthVal, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, DepthVal, _>::from((&mut tree.0, iter, ptr))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(
            values,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(|x| x.to_string())
        );
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = BfsIterPtr::<_, DepthVal, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, DepthVal, _>::from((&mut tree.0, iter, ptr))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [3, 6, 7, 9, 10, 11].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [0, 1, 1, 2, 2, 2]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let n7 = n3.get_child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = BfsIterPtr::<_, DepthVal, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, DepthVal, _>::from((&mut tree.0, iter, ptr))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [7, 10, 11].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [0, 1, 1]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn bfs_into_iter_sibling_idx() {
    {
        let mut tree = tree();
        let mut stack = VecDeque::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = BfsIterPtr::<_, SiblingIdxVal, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, SiblingIdxVal, _>::from((&mut tree.0, iter, ptr))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(
            values,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(|x| x.to_string())
        );
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = BfsIterPtr::<_, SiblingIdxVal, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, SiblingIdxVal, _>::from((&mut tree.0, iter, ptr))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [3, 6, 7, 9, 10, 11].map(|x| x.to_string()));
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 0, 1]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let n7 = n3.get_child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = BfsIterPtr::<_, SiblingIdxVal, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, SiblingIdxVal, _>::from((&mut tree.0, iter, ptr))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.1.clone()).collect();
        assert_eq!(values, [7, 10, 11].map(|x| x.to_string()));
        let sibling: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(sibling, [0, 0, 1]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}

#[test]
fn bfs_into_iter_depth_sibling_idx() {
    {
        let mut tree = tree();
        let mut stack = VecDeque::default();

        let root = tree.root();
        let ptr = root.node_ptr().clone();
        let iter = BfsIterPtr::<_, DepthSiblingIdxVal, _>::from((&mut stack, ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.2.clone()).collect();
        assert_eq!(
            values,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11].map(|x| x.to_string())
        );
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [0, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1]);

        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
        assert_eq!(tree.get_root(), None);
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let ptr = n3.node_ptr().clone();
        let iter = BfsIterPtr::<_, DepthSiblingIdxVal, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.2.clone()).collect();
        assert_eq!(values, [3, 6, 7, 9, 10, 11].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [0, 1, 1, 2, 2, 2]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 1, 0, 0, 1]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 5);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 4, 5, 8].map(|x| x.to_string()));
    }

    {
        let mut tree = tree();

        let root = tree.root();
        let n3 = root.get_child(1).unwrap();
        let n7 = n3.get_child(1).unwrap();
        let ptr = n7.node_ptr().clone();
        let iter = BfsIterPtr::<_, DepthSiblingIdxVal, _>::from((VecDeque::default(), ptr.clone()));
        let iter = unsafe {
            BfsIterInto::<_, Auto, SplitRecursive, DepthSiblingIdxVal, _>::from((
                &mut tree.0,
                iter,
                ptr,
            ))
        };
        let result: Vec<_> = iter.collect();
        let values: Vec<_> = result.iter().map(|x| x.2.clone()).collect();
        assert_eq!(values, [7, 10, 11].map(|x| x.to_string()));
        let depths: Vec<_> = result.iter().map(|x| x.0).collect();
        assert_eq!(depths, [0, 1, 1]);
        let sibling: Vec<_> = result.iter().map(|x| x.1).collect();
        assert_eq!(sibling, [0, 0, 1]);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 8);
        assert_eq!(
            tree.get_root().map(|x| x.data().clone()),
            Some(1.to_string())
        );
        let values: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
        assert_eq!(values, [1, 2, 3, 4, 5, 6, 8, 9].map(|x| x.to_string()));
    }
}
