#![cfg(feature = "serde")]
use orx_tree::*;

#[test]
fn serialize() {
    let tree = BinaryTree::<i32>::empty();
    let json = serde_json::to_string(&tree).unwrap();
    assert_eq!(json, "[]");

    let tree = DynTree::new(10);
    let json = serde_json::to_string(&tree).unwrap();
    assert_eq!(json, "[[0,10]]");

    //      0
    //     ╱ ╲
    //    ╱   ╲
    //   1     2
    //  ╱     ╱ ╲
    // 3     4   5
    // |         |
    // 6         7
    let mut tree = DaryTree::<4, _>::new(0);
    let [id1, id2] = tree.root_mut().push_children([1, 2]);
    let id3 = tree.node_mut(id1).push_child(3);
    tree.node_mut(id3).push_child(6);
    let [_, id5] = tree.node_mut(id2).push_children([4, 5]);
    tree.node_mut(id5).push_child(7);

    let json = serde_json::to_string(&tree).unwrap();
    assert_eq!(json, "[[0,0],[1,1],[2,3],[3,6],[1,2],[2,4],[2,5],[3,7]]");
}

#[test]
fn deserialize() {
    let json = "[]";
    let result: Result<DaryTree<4, i32>, _> = serde_json::from_str(json);
    let tree = result.unwrap();
    assert!(tree.is_empty());

    let json = "[[0,10]]";
    let result: Result<DynTree<i32>, _> = serde_json::from_str(json);
    let tree = result.unwrap();
    assert_eq!(tree.len(), 1);
    assert_eq!(tree.root().data(), &10);

    //      0
    //     ╱ ╲
    //    ╱   ╲
    //   1     2
    //  ╱     ╱ ╲
    // 3     4   5
    // |         |
    // 6         7
    let json = "[[0, 0], [1, 1], [2, 3], [3, 6], [1, 2], [2, 4], [2, 5], [3, 7]]";
    let result: Result<BinaryTree<i32>, _> = serde_json::from_str(json);
    let tree = result.unwrap();
    let bfs_values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    assert_eq!(bfs_values, [0, 1, 2, 3, 4, 5, 6, 7]);

    // errors

    // A. First element of DepthFirstSequence (root of the tree) must have depth 0;
    // however, received a depth of 1.
    let json = "[[1,10]]";
    let result: Result<DynTree<i32>, _> = serde_json::from_str(json);
    assert!(result.is_err());

    // B. Let d1 and d2 be two consecutive depths in the depth-first sequence.
    // Then, (i) d2=d1+1, (ii) d2=d1 or (iii) d2<d1 are valid cases.
    // However, received the invalid case where d2>d1+1 (d1=1, d2=3).
    let json = "[[0, 0], [1, 1], [3, 6], [1, 2], [2, 4], [2, 5], [3, 7]]";
    let result: Result<BinaryTree<i32>, _> = serde_json::from_str(json);
    assert!(result.is_err());
}
