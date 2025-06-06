use orx_tree::*;

fn main() {
    // build tree
    let mut tree = DynTree::new(1);

    let mut root = tree.root_mut();
    let [id2, id3] = root.push_children([2, 3]);
    let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
    tree.node_mut(&id4).push_child(8);
    let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
    tree.node_mut(&id6).push_child(9);
    tree.node_mut(&id7).push_children([10, 11]);

    print!("{}", &tree);
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

    // A. depth-first node values from root
    let root = tree.root();
    println!(
        "depth-first node values from root: {:?}",
        root.walk::<Dfs>().collect::<Vec<_>>()
    );

    // B. breadth-first node values from node 3
    let n3 = tree.node(&id3);
    println!(
        "breadth-first node values from root: {:?}",
        n3.walk::<Bfs>().collect::<Vec<_>>()
    );

    // C. post-order node values from node 2
    let n2 = tree.node(&id2);
    println!(
        "post-order node values from root: {:?}",
        n2.walk::<PostOrder>().collect::<Vec<_>>()
    );

    // using traversal over and over again to minimize allocation
    let mut t = Traversal.dfs(); // depth-first traverser over data
    assert_eq!(
        tree.root().walk_with(&mut t).copied().collect::<Vec<_>>(),
        [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]
    );
    assert_eq!(
        tree.node(&id2)
            .walk_with(&mut t)
            .copied()
            .collect::<Vec<_>>(),
        [2, 4, 8, 5]
    );
    assert_eq!(
        tree.node(&id3)
            .walk_with(&mut t)
            .copied()
            .collect::<Vec<_>>(),
        [3, 6, 9, 7, 10, 11]
    );

    // using traversal to traverse over nodes, rather than only data, with access to children and parent
    let mut t = Traversal.bfs().over_nodes(); // breadth-first traverser over nodes
    let x: Vec<_> = tree
        .node(&id3)
        .walk_with(&mut t)
        .map(|node| {
            let node_value = *node.data();
            let children_values_sum = node.children().map(|x| x.data()).sum::<u64>();
            (node_value, children_values_sum)
        })
        .collect();
    println!(
        "breadth-first (node value, sum of children values) pairs from n3: {:?}",
        &x
    );

    // using traversal to additionally access to depth and sibling indices
    let mut t = Traversal.dfs().with_depth();
    let n2 = tree.node(&id2);
    println!(
        "depth-first (depth, node value) pairs from n2: {:?}",
        n2.walk_with(&mut t).collect::<Vec<_>>()
    );

    let mut t = Traversal.dfs().with_depth().with_sibling_idx();
    let n3 = tree.node(&id3);
    println!(
        "depth-first (depth, sibling index, node value) tuples from n3: {:?}",
        n3.walk_with(&mut t).collect::<Vec<_>>()
    );
}
