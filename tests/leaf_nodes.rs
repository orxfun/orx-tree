// tests on nodes without any children

use orx_tree::*;

/// https://github.com/orxfun/orx-tree/issues/183
#[test]
fn leaf_into_new_tree() {
    let mut tree = DynTree::new(0);
    let idx = tree.root_mut().push_child(1);

    let tree2: DynTree<_> = tree.node_mut(idx).into_new_tree();

    assert_eq!(tree2.len(), 1);
    assert_eq!(
        tree2.root().walk::<Dfs>().copied().collect::<Vec<_>>(),
        vec![1]
    );
}

#[test]
fn root_into_new_tree() {
    let mut tree = DynTree::new(0);

    let tree2: DynTree<_> = tree.root_mut().into_new_tree();

    assert_eq!(tree2.len(), 1);
    assert_eq!(
        tree2.root().walk::<Dfs>().copied().collect::<Vec<_>>(),
        vec![0]
    );
}

#[test]
fn leaf_into_walk() {
    let mut tree = DynTree::new(0);
    let idx = tree.root_mut().push_child(1);

    let values: Vec<_> = tree.node_mut(idx).into_walk::<Dfs>().collect();
    assert_eq!(values, vec![1]);

    let remaining: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    assert_eq!(remaining, vec![0]);
}

#[test]
fn leaf_into_leaves() {
    let mut tree = DynTree::new(0);
    let idx = tree.root_mut().push_child(1);

    let values: Vec<_> = tree.node_mut(idx).into_leaves::<Dfs>().collect();
    assert_eq!(values, vec![1]);

    let remaining: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    assert_eq!(remaining, vec![0]);
}

#[test]
fn leaf_into_subtree() {
    let mut tree = DynTree::new(0);
    let idx = tree.root_mut().push_child(1);

    let subtree = tree.node_mut(idx).into_subtree();
    let mut tree2 = DynTree::new(42);
    tree2.root_mut().push_child_tree(subtree);
    let values: Vec<_> = tree2.root().walk::<Dfs>().copied().collect();
    assert_eq!(values, vec![42, 1]);

    let remaining: Vec<_> = tree.root().walk::<Dfs>().copied().collect();
    assert_eq!(remaining, vec![0]);
}
