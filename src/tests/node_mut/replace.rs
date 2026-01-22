use crate::tests::node_mut::utils::{
    collect_sorted_subtree, get_main_tree, get_main_tree_copy, get_other_tree, get_other_tree_copy,
};
use crate::*;
use std::vec::Vec;

fn expected_nodes<T: Clone + Eq + Ord>(
    initial_nodes: &[T],
    expected_removed: &[T],
    expected_inserted: &[T],
) -> Vec<T> {
    let mut expected_nodes: Vec<_> = initial_nodes
        .iter()
        .filter(|x| !expected_removed.contains(x))
        .cloned()
        .collect();

    expected_nodes.extend_from_slice(expected_inserted);

    expected_nodes.sort();

    expected_nodes
}

#[test]
fn replace_with_cloned() {
    let tree = get_main_tree().into_lazy_reclaim();
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

            let root_value = other.node(id_src).data().clone();
            let subtree = other.node(id_src).as_cloned_subtree();
            let (idx, removed) = tree.node_mut(id_dst).replace_with::<Bfs, _>(subtree);
            let removed: Vec<_> = removed.collect();

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, expected_nodes);

            assert_eq!(removed, expected_removed);
            assert_eq!(tree.node(idx).data(), &root_value);
        }
    }
}

#[test]
fn replace_with_copied() {
    let tree = get_main_tree_copy().into_lazy_reclaim();
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

            let root_value = other.node(id_src).data().clone();
            let subtree = other.node(id_src).as_copied_subtree();
            let (idx, removed) = tree.node_mut(id_dst).replace_with::<Bfs, _>(subtree);
            let removed: Vec<_> = removed.collect();

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, expected_nodes);

            assert_eq!(removed, expected_removed);
            assert_eq!(tree.node(idx).data(), &root_value);
        }
    }
}

#[test]
fn replace_with_moved() {
    let tree = get_main_tree().into_lazy_reclaim();
    let initial_nodes = collect_sorted_subtree(tree.root());

    let other = get_other_tree();

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

            let root_value = other.node(id_src).data().clone();
            let subtree = other.node_mut(id_src).into_subtree();
            let (idx, removed) = tree.node_mut(id_dst).replace_with::<Bfs, _>(subtree);
            let removed: Vec<_> = removed.collect();

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, expected_nodes);

            assert_eq!(removed, expected_removed);
            assert_eq!(tree.node(idx).data(), &root_value);
        }
    }
}
