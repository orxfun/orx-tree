// tests on nodes without any children

use orx_tree::*;

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

/// https://github.com/orxfun/orx-tree/issues/183
#[test]
fn leaf_into_new_tree() {
    let mut tree = DynTree::new(0);
    tree.root_mut().push_child(1);

    let tree2: DynTree<_> = tree
        .root_mut()
        .children_mut()
        .nth(0)
        .unwrap()
        .into_new_tree();

    assert_eq!(tree2.len(), 1);
    assert_eq!(
        tree2.root().walk::<Dfs>().copied().collect::<Vec<_>>(),
        vec![1]
    );
}
