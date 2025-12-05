use orx_tree::traversal::*;
use orx_tree::*;

fn find_node<'a, V: TreeVariant>(tree: &'a Tree<V>, predicate: &V::Item) -> Option<Node<'a, V>>
where
    V::Item: Eq,
{
    let mut traverser = Dfs::<OverNode>::new();
    let node = {
        let root_node = tree.get_root().unwrap();
        let mut walker = root_node.walk_with(&mut traverser);
        let x = walker.find(|v| v.data() == predicate);
        // drop(walker);
        // drop(traverser);
        x
    };

    // ERROR: cannot return value referencing local variable `traverser`
    return node;
    None
}
