use orx_tree::traversal::*;
use orx_tree::*;

/*
These tests are created with respect to the issue: https://github.com/orxfun/orx-tree/issues/188.

They are expected to compile.

The `find` methods below return a node referencing the `tree`, which must be independent
of the root node or traverser if provided externally.

The methods must NOT fail to compile due to the following errors:
* referencing local variable `traverser`, or
* referencing local variable `root`.

These are temporary references, instead the output's lifetime must depend on the tree.
*/

fn find_ancestors<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.ancestors().find(|x| x.data() == predicate)
}

#[cfg(feature = "parallel")]
fn find_ancestors_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.ancestors_par().find(|x| x.data() == predicate)
}

fn find_children<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.children().find(|x| x.data() == predicate)
}

#[cfg(feature = "parallel")]
fn find_children_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.children_par().find(|x| x.data() == predicate)
}

fn find_custom_walk<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let custom_walk = |node: Node<'a, V>| node.get_child(0);
    let root = tree.get_root()?;
    let mut walker = root.custom_walk(custom_walk);
    walker.find(|v| v == &predicate)
}

#[cfg(feature = "parallel")]
fn find_custom_walk_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let custom_walk = |node: Node<'a, V>| node.get_child(0);
    let root = tree.get_root()?;
    let walker = root.custom_walk_par(custom_walk);
    walker.find(|v| v == &predicate)
}

fn find_walk<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    let mut walker = root.walk::<Dfs>();
    walker.find(|v| v == &predicate)
}

#[cfg(feature = "parallel")]
fn find_walk_par<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    let walker = root.walk_par::<Dfs>();
    walker.find(|v| v == &predicate)
}

fn find_walk_with<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    let mut walker = root.walk_with(&mut traverser);
    walker.find(|v| v.data() == predicate)
}

#[cfg(feature = "parallel")]
fn find_walk_with_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    let walker = root.walk_with_par(&mut traverser);
    walker.find(|v| v.data() == predicate)
}

fn find_paths<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.paths::<Dfs>()
        .find(|x| x.clone().collect::<Vec<_>>().contains(&predicate))
        .and_then(|mut x| x.next())
}

fn find_paths_with<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    root.paths_with(&mut traverser)
        .find(|x| {
            x.clone()
                .map(|x| x.data())
                .collect::<Vec<_>>()
                .contains(&predicate)
        })
        .and_then(|mut x| x.next())
}

#[cfg(feature = "parallel")]
fn find_paths_with_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    root.paths_with_par(&mut traverser)
        .find(|x| {
            x.clone()
                .map(|x| x.data())
                .collect::<Vec<_>>()
                .contains(&predicate)
        })
        .and_then(|mut x| x.next())
}

#[cfg(feature = "parallel")]
fn find_paths_par<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.paths_par::<Dfs>()
        .find(|x| x.clone().collect::<Vec<_>>().contains(&predicate))
        .and_then(|mut x| x.next())
}

fn find_leaves<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.leaves::<Dfs>().find(|v| v == &predicate)
}

#[cfg(feature = "parallel")]
fn find_leaves_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<&'a V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.leaves_par::<Dfs>().find(|v| v == &predicate)
}

fn find_leaves_with<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    root.leaves_with(&mut traverser)
        .find(|v| v.data() == predicate)
}

#[cfg(feature = "parallel")]
fn find_leaves_with_par<'a, V: TreeVariant>(
    tree: &'a Tree<V>,
    predicate: &V::Item,
) -> Option<Node<'a, V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    root.leaves_with_par(&mut traverser)
        .find(|v| v.data() == predicate)
}

fn find_indices<V: TreeVariant>(tree: &Tree<V>, predicate: &V::Item) -> Option<NodeIdx<V>>
where
    V::Item: Eq + Sync + Send,
{
    let root = tree.get_root()?;
    root.indices::<Dfs>()
        .find(|v| tree.node(*v).data() == predicate)
}

fn find_indices_with<V: TreeVariant>(tree: &Tree<V>, predicate: &V::Item) -> Option<NodeIdx<V>>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverNode>::new();
    let root = tree.get_root()?;
    root.indices_with(&mut traverser)
        .find(|v| tree.node(*v).data() == predicate)
}

// mut

fn find_custom_walk_mut<'a, V: TreeVariant>(
    tree: &'a mut Tree<V>,
    predicate: &V::Item,
) -> Option<&'a mut V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let custom_walk = |node: Node<'a, V>| node.get_child(0);
    let mut root = tree.get_root_mut()?;
    let mut walker = root.custom_walk_mut(custom_walk);
    walker.find(|v| v == &predicate)
}

fn find_walk_mut<'a, V: TreeVariant>(
    tree: &'a mut Tree<V>,
    predicate: &V::Item,
) -> Option<&'a mut V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let mut root = tree.get_root_mut()?;
    let mut walker = root.walk_mut::<Dfs>();
    walker.find(|v| v == &predicate)
}

fn find_leaves_mut<'a, V: TreeVariant>(
    tree: &'a mut Tree<V>,
    predicate: &V::Item,
) -> Option<&'a mut V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let mut root = tree.get_root_mut()?;
    root.leaves_mut::<Dfs>().find(|v| v == &predicate)
}

fn find_leaves_mut_with<'a, V: TreeVariant>(
    tree: &'a mut Tree<V>,
    predicate: &V::Item,
) -> Option<&'a mut V::Item>
where
    V::Item: Eq + Sync + Send,
{
    let mut traverser = Dfs::<OverData>::new();
    let mut root = tree.get_root_mut()?;
    root.leaves_mut_with(&mut traverser)
        .find(|v| v == &predicate)
}

#[test]
fn node_ref_lifetime_tests() {
    let mut tree = DynTree::new(42);

    assert_eq!(find_ancestors(&tree, &7), None);
    assert_eq!(find_children(&tree, &7), None);
    assert_eq!(find_custom_walk(&tree, &7), None);
    assert_eq!(find_walk(&tree, &7), None);
    assert_eq!(find_walk_with(&tree, &7), None);
    assert_eq!(find_paths(&tree, &7), None);
    assert_eq!(find_paths_with(&tree, &7), None);
    assert_eq!(find_leaves(&tree, &7), None);
    assert_eq!(find_leaves_with(&tree, &7), None);
    assert_eq!(find_indices(&tree, &7), None);
    assert_eq!(find_indices_with(&tree, &7), None);
    #[cfg(feature = "parallel")]
    {
        assert_eq!(find_ancestors_par(&tree, &7), None);
        assert_eq!(find_children_par(&tree, &7), None);
        assert_eq!(find_custom_walk_par(&tree, &7), None);
        assert_eq!(find_walk_par(&tree, &7), None);
        assert_eq!(find_walk_with_par(&tree, &7), None);
        assert_eq!(find_paths_par(&tree, &7), None);
        assert_eq!(find_paths_with_par(&tree, &7), None);
        assert_eq!(find_leaves_par(&tree, &7), None);
        assert_eq!(find_leaves_with_par(&tree, &7), None);
    }

    assert_eq!(find_custom_walk_mut(&mut tree, &7), None);
    assert_eq!(find_walk_mut(&mut tree, &7), None);
    assert_eq!(find_leaves_mut(&mut tree, &7), None);
    assert_eq!(find_leaves_mut_with(&mut tree, &7), None);
}
