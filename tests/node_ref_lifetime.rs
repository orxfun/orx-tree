use orx_tree::traversal::*;
use orx_tree::*;

/*
These tests are created wrt the issue: https://github.com/orxfun/orx-tree/issues/188.

They are expected to compile.
*/

/// This method returns a Node that is referencing the `tree`.
///
/// It must not fail to compile by due to errors:
/// * referencing local variable `traverser`, or
/// * referencing local variable `root_node`.
fn find_walk_with<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<Node<'a, V>>
where
    V::Item: Eq,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root_node = tree.get_root()?;
    let mut walker = root_node.walk_with(&mut traverser);
    walker.find(|v| v.data() == predicate)
}

#[test]
fn node_ref_lifetime_tests() {
    let tree = DynTree::new(42);

    let node = find_walk_with(&tree, &33);
    assert_eq!(node, None);
}
