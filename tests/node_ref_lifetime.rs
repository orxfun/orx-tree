use orx_tree::traversal::*;
use orx_tree::*;

/*
These tests are created wrt the issue: https://github.com/orxfun/orx-tree/issues/188.

They are expected to compile.

The `find` methods below return a node referencing the `tree`, which must be independent
of the root node or traverser if provided externally.

The methods must NOT fail to compile due to the following errors:
* referencing local variable `traverser`, or
* referencing local variable `root_node`.

These are temporary references. The output's lifetime must not depend on these temporary values.
*/

fn find_walk<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq,
{
    let root_node = tree.get_root()?;
    let mut walker = root_node.walk::<Dfs>();
    walker.find(|v| v == &predicate)
}

fn find_walk_par<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root_node = tree.get_root()?;
    let walker = root_node.walk_par::<Dfs>();
    walker.find(|v| v == &predicate)
}

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

    let node = find_walk(&tree, &33);
    assert_eq!(node, None);

    let node = find_walk_par(&tree, &33);
    assert_eq!(node, None);

    let node = find_walk_with(&tree, &33);
    assert_eq!(node, None);
}
