use crate::tests::node_mut::utils::{collect_sorted_subtree, get_main_tree};
use crate::*;

#[test]
fn swap_subtrees() {
    let tree = get_main_tree();
    let initial_nodes = collect_sorted_subtree(tree.root());

    for i in 0..tree.len() {
        for j in 0..tree.len() {
            let mut tree = tree.clone();
            let id_a = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_b = tree.root().indices::<Bfs>().nth(j).unwrap();
            if tree.node(id_a).is_ancestor_of(id_b) || tree.node(id_b).is_ancestor_of(id_a) {
                continue;
            }
            tree.swap_subtrees(id_a, id_b);
            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, initial_nodes);
        }
    }
}

#[test]
fn try_swap_nodes() {
    let tree = get_main_tree();
    let initial_nodes = collect_sorted_subtree(tree.root());

    for i in 0..tree.len() {
        for j in 0..tree.len() {
            let mut tree = tree.clone();
            let id_a = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_b = tree.root().indices::<Bfs>().nth(j).unwrap();
            if tree.node(id_a).is_ancestor_of(id_b) || tree.node(id_b).is_ancestor_of(id_a) {
                continue;
            }
            let swapped = tree.try_swap_nodes(id_a, id_b).is_ok();
            assert!(swapped);

            let nodes = collect_sorted_subtree(tree.root());
            assert_eq!(nodes, initial_nodes);
        }
    }
}

#[test]
fn try_swap_nodes_fail() {
    let original_tree = get_main_tree();

    for i in 0..original_tree.len() {
        for j in 0..original_tree.len() {
            let mut tree = original_tree.clone();
            let id_a = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_b = tree.root().indices::<Bfs>().nth(j).unwrap();
            if !tree.node(id_a).is_ancestor_of(id_b) && !tree.node(id_b).is_ancestor_of(id_a) {
                continue;
            }
            let swapped = tree.try_swap_nodes(id_a, id_b).is_ok();
            assert!(!swapped);

            assert_eq!(&tree, &original_tree);
        }
    }
}
