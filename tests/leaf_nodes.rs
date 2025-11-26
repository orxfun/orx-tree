// tests on nodes without any children

use orx_tree::*;

/// https://github.com/orxfun/orx-tree/issues/183
#[test]
fn leaf_into_new_tree() {
    let mut tree = DynTree::new(0);
    tree.root_mut().push_child(1);

    // Does not panic if we add a children to 1
    // tree.root_mut().children_mut().next().unwrap().push_child(2);
    let tree2: DynTree<_> = tree
        .root_mut()
        .children_mut()
        .nth(0)
        .unwrap()
        .into_new_tree();
}
