use crate::*;
use std::{
    dbg, println,
    string::{String, ToString},
    vec::Vec,
};

fn to_str<const N: usize>(values: [i32; N]) -> [String; N] {
    values.map(|x| x.to_string())
}

fn get_main_tree() -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let [id1, id2] = tree.root_mut().push_children(to_str([1, 2]));
    let [_id3, _id4, _id5] = tree.node_mut(id1).push_children(to_str([3, 4, 5]));
    let [id6] = tree.node_mut(id2).push_children(to_str([6]));
    let [_id7, _id8] = tree.node_mut(id6).push_children(to_str([7, 8]));
    tree
}

fn get_other_tree() -> DaryTree<4, String> {
    let mut tree = DaryTree::new(10.to_string());
    let [id11, _id12] = tree.root_mut().push_children(to_str([11, 12]));
    let [_id13, _id14] = tree.node_mut(id11).push_children(to_str([13, 14]));
    tree
}

/// Subtrees can be pushed as child trees of any node when cloned,
/// no potential panic.
#[test]
fn push_child_tree_cloned() {
    let tree = get_main_tree();
    let initial_nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();

    let other = get_other_tree();
    let ids_other: Vec<_> = other.root().indices::<Bfs>().collect();

    for i in 0..tree.len() {
        for id_other in ids_other.iter().copied() {
            let mut tree = tree.clone();
            let id = tree.root().indices::<Bfs>().nth(i).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(other.node(id_other).walk::<Bfs>().cloned());
            expected_nodes.sort();

            let subtree = other.node(id_other).as_cloned_subtree();
            tree.node_mut(id).push_child_tree(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}

/// Subtrees can be pushed as child trees of any node when moved,
/// no potential panic.
#[test]
fn push_child_tree_moved() {
    let tree = get_main_tree();
    let initial_nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
    let other = get_other_tree();

    for i in 0..tree.len() {
        for j in 0..other.len() {
            let mut tree = tree.clone();
            let mut other = other.clone();
            let id = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_other = other.root().indices::<Bfs>().nth(j).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(other.node(id_other).walk::<Bfs>().cloned());
            expected_nodes.sort();

            let subtree = other.node_mut(id_other).into_subtree();
            tree.node_mut(id).push_child_tree(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}
