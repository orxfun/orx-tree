use orx_iterable::{IntoCloningIterable, Iterable};
use orx_tree::*;

fn main() {
    // build tree
    let mut tree = DynTree::new(1);

    let mut root = tree.root_mut();
    let [id2, id3] = root.push_children([2, 3]);
    let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    let id8 = tree.node_mut(id4).push_child(8);
    let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    tree.node_mut(id6).push_child(9);
    let [_, id11] = tree.node_mut(id7).push_children([10, 11]);

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

    let root = tree.root();
    let n3 = tree.node(id3);
    let n8 = tree.node(id8);
    let n11 = tree.node(id11);

    // leaves

    println!("\nleaves");

    println!(
        "leaves from root in depth-first order:   {:?}",
        root.leaves::<Dfs>().collect::<Vec<_>>()
    );

    println!(
        "leaves from root in breadth-first order: {:?}",
        root.leaves::<Bfs>().collect::<Vec<_>>()
    );

    println!(
        "leaves from root in post-order:          {:?}",
        root.leaves::<PostOrder>().collect::<Vec<_>>()
    );

    println!(
        "leaves from n3 in depth-first order: {:?}",
        n3.leaves::<Dfs>().collect::<Vec<_>>()
    );

    // leaves_with

    let mut t = Traversal.dfs().over_nodes();
    println!(
        "leaves and their parents from root in depth-first order: {:?}",
        root.leaves_with(&mut t)
            .map(|leaf_node| {
                let data = *leaf_node.data();
                let parent_data = *leaf_node.parent().unwrap().data();
                (data, parent_data)
            })
            .collect::<Vec<_>>()
    );

    // paths

    println!("\npaths");

    println!(
        "all (reversed) paths from root in depth-first order:   {:?}",
        root.paths::<Dfs>()
            .map(|path| path.collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );

    println!(
        "all (reversed) paths from root in breadth-first order: {:?}",
        root.paths::<Bfs>()
            .map(|path| path.collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );

    // cheap-convert path into orx_iterable::Iterable in order to iterate over
    // each path multiple times without requiring to allocate & collect them.
    println!(
        "maximum-value-sum path that does not contain node 7: {:?}",
        root.paths::<Dfs>()
            .map(|path| path.into_iterable())
            .filter(|path| path.iter().all(|x| *x != 7))
            .max_by_key(|path| path.iter().sum::<u32>())
            .map(|path| path.iter().collect::<Vec<_>>())
    );

    // ancestors

    println!("\nancestors");

    println!(
        "ancestors of the root: {:?}",
        root.ancestors().map(|node| node.data()).collect::<Vec<_>>()
    );

    println!(
        "ancestors of node 3: {:?}",
        n3.ancestors().map(|node| node.data()).collect::<Vec<_>>()
    );

    println!(
        "ancestors of node 11: {:?}",
        n11.ancestors().map(|node| node.data()).collect::<Vec<_>>()
    );

    println!(
        "node information (rather than only data) of ancestors of node 8: {:?}",
        n8.ancestors().collect::<Vec<_>>()
    );
}
