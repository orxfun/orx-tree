use orx_tree::*;

/// ```
///      1
///     ╱ ╲
///    ╱   ╲
///   2     3
///  ╱ ╲   ╱ ╲
/// 4   5 6   7
/// |     |  ╱ ╲
/// 8     9 10  11
/// ```
fn tree() -> DynTree<i32> {
    let mut tree = DynTree::new(1);

    let mut root = tree.root_mut();
    let [id2, id3] = root.push_children([2, 3]);
    let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    let _ = tree.node_mut(id4).push_child(8);
    let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    let _ = tree.node_mut(id6).push_child(9);
    tree.node_mut(id7).push_children([10, 11]);
    tree
}

#[test]
fn send_node_indices() {
    let tree = tree();
    let indices: Vec<_> = tree.root().indices::<Bfs>().collect();

    std::thread::scope(|s| {
        let indices = indices.as_slice();
        let tree = &tree;
        for _ in 0..4 {
            s.spawn(|| {
                for (i, idx) in indices.iter().copied().enumerate() {
                    let node = tree.node(idx);
                    assert_eq!(*node.data(), i as i32 + 1);
                }
            });
        }
    })
}

#[test]
fn sync_node_indices() {
    let tree = tree();

    let indices: Vec<_> = tree.root().indices::<Bfs>().collect();
    let index_refs: Vec<_> = indices.iter().collect();

    std::thread::scope(|s| {
        let index_refs = index_refs.as_slice();
        let tree = &tree;
        for _ in 0..4 {
            s.spawn(|| {
                for (i, idx) in index_refs.iter().copied().enumerate() {
                    let node = tree.node(*idx);
                    assert_eq!(*node.data(), i as i32 + 1);
                }
            });
        }
    })
}
