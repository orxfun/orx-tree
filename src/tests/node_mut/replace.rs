use crate::tests::node_mut::utils::{
    collect_sorted_subtree, get_main_tree, get_main_tree_copy, get_other_tree, get_other_tree_copy,
    to_str,
};
use crate::*;
use std::string::{String, ToString};
use std::vec::Vec;
use test_case::test_matrix;

fn expected_nodes<T: Clone + Eq + Ord>(
    initial_nodes: &[T],
    expected_removed: &[T],
    expected_inserted: &[T],
) -> Vec<T> {
    let mut expected_nodes: Vec<_> = initial_nodes
        .iter()
        .cloned()
        .filter(|x| !expected_removed.contains(x))
        .collect();

    expected_nodes.extend_from_slice(expected_inserted);

    expected_nodes.sort();

    expected_nodes
}

#[test]
fn replace_with_cloned() {
    let tree = get_main_tree();
    let initial_nodes = collect_sorted_subtree(tree.root());

    let other = get_other_tree();
    let ids_other: Vec<_> = other.root().indices::<Bfs>().collect();

    for i in 0..tree.len() {
        for id_src in ids_other.iter().copied() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();

            let expected_removed = collect_sorted_subtree(tree.node(id_dst));
            let expected_inserted = collect_sorted_subtree(other.node(id_src));
            let expected_nodes =
                expected_nodes(&initial_nodes, &expected_removed, &expected_inserted);

            let subtree = other.node(id_src).as_cloned_subtree();
            let removed = tree.node_mut(id_dst).replace::<Bfs, _>(subtree);
            let removed: Vec<_> = removed.collect();

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, expected_nodes);

            assert_eq!(removed, expected_removed);
        }
    }
}

#[test]
fn replace_with_copied() {
    let tree = get_main_tree_copy();
    let initial_nodes = collect_sorted_subtree(tree.root());

    let other = get_other_tree_copy();
    let ids_other: Vec<_> = other.root().indices::<Bfs>().collect();

    for i in 0..tree.len() {
        for id_src in ids_other.iter().copied() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();

            let expected_removed = collect_sorted_subtree(tree.node(id_dst));
            let expected_inserted = collect_sorted_subtree(other.node(id_src));
            let expected_nodes =
                expected_nodes(&initial_nodes, &expected_removed, &expected_inserted);

            let subtree = other.node(id_src).as_copied_subtree();
            let removed = tree.node_mut(id_dst).replace::<Bfs, _>(subtree);
            let removed: Vec<_> = removed.collect();

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, expected_nodes);

            assert_eq!(removed, expected_removed);
        }
    }
}

#[test]
fn replace_with_moved() {
    let tree = get_main_tree_copy();
    let initial_nodes = collect_sorted_subtree(tree.root());

    let other = get_other_tree_copy();

    for i in 0..tree.len() {
        for j in 0..other.len() {
            let mut tree = tree.clone();
            let mut other = other.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_src = other.root().indices::<Bfs>().nth(j).unwrap();

            let expected_removed = collect_sorted_subtree(tree.node(id_dst));
            let expected_inserted = collect_sorted_subtree(other.node(id_src));
            let expected_nodes =
                expected_nodes(&initial_nodes, &expected_removed, &expected_inserted);

            let subtree = other.node_mut(id_src).into_subtree();
            let removed = tree.node_mut(id_dst).replace::<Bfs, _>(subtree);
            let removed: Vec<_> = removed.collect();

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, expected_nodes);

            assert_eq!(removed, expected_removed);
        }
    }
}
