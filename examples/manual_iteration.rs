use orx_tree::*;

fn main() {
    // build tree
    let mut tree = DynTree::new(1);

    // A. build the tree with mutable references

    let mut root = tree.root_mut();
    root.push_child(2);
    let mut n2 = root.child_mut(0);
    n2.push_children([4, 5]);
    n2.child_mut(0).push_child(8);

    root.push_child(3);
    let mut n3 = root.child_mut(1);
    n3.push_children([6, 7]);
    n3.child_mut(0).push_child(9);
    n3.child_mut(1).push_children([10, 11]);

    println!("building the tree with manual iteration:\n{}", &tree);
    // 1
    // ├──2
    // │  ├──4
    // │  │  └──8
    // │  └──5
    // └──3
    //    ├──6
    //    │  └──9
    //    └──7
    //       ├──10
    //       └──11

    // B. custom iterator

    fn next_node<'a, T>(node: DynNode<'a, T>) -> Option<DynNode<'a, T>> {
        let sibling_idx = node.sibling_idx();
        let is_last_sibling = sibling_idx == node.num_siblings() - 1;

        match is_last_sibling {
            true => node.get_child(0),
            false => match node.parent() {
                Some(parent) => {
                    let child_idx = sibling_idx + 1;
                    parent.get_child(child_idx)
                }
                None => None,
            },
        }
    }

    // manual loop over the custom next_node
    let mut values = vec![];
    let mut current = tree.root();
    values.push(*current.data());
    while let Some(node) = next_node(current) {
        values.push(*node.data());
        current = node;
    }
    println!(
        "\na custom iterator:\n* start from a node (root here)\n* move to next sibling if any\n* move to first child otherwise\n=> {:?}\n",
        &values
    );
    assert_eq!(values, [1, 2, 3, 6, 7, 10, 11]);

    // or simply use `custom_walk`
    let root = tree.root();
    let values: Vec<_> = root.custom_walk(next_node).copied().collect();
    assert_eq!(values, [1, 2, 3, 6, 7, 10, 11]);

    // B. mutate the structure of the tree with manual mutable traversal
    let root = tree.root_mut();
    let n3 = root.into_child_mut(1).unwrap();
    let mut n6 = n3.into_child_mut(0).unwrap();
    n6.push_child(12);
    let n3 = n6.into_parent_mut().unwrap();
    let n7 = n3.into_child_mut(1).unwrap();
    n7.prune();

    print!(
        "mutating structure of the tree with manual iteration:\n{}",
        &tree
    );
    // 1
    // ├──2
    // │  ├──4
    // │  │  └──8
    // │  └──5
    // └──3
    //    └──6
    //       ├──9
    //       └──12
}
