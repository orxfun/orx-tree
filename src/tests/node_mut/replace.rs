use crate::tests::node_mut::utils::{
    get_main_tree, get_main_tree_copy, get_other_tree, get_other_tree_copy, to_str,
};
use crate::*;
use std::string::ToString;
use std::vec::Vec;
use test_case::test_matrix;

#[test]
fn replace_with_cloned() {
    let tree = get_main_tree();
    let initial_nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();

    let other = get_other_tree();
    let ids_other: Vec<_> = other.root().indices::<Bfs>().collect();

    for i in 0..tree.len() {
        for id_src in ids_other.iter().copied() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();

            let mut expected_removed: Vec<_> = tree.node(id_dst).walk::<Bfs>().cloned().collect();
            expected_removed.sort();

            let mut expected_inserted: Vec<_> = other.node(id_src).walk::<Bfs>().cloned().collect();
            expected_inserted.sort();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(other.node(id_src).walk::<Bfs>().cloned());
            expected_nodes.sort();

            let subtree = other.node(id_src).as_cloned_subtree();
            tree.node_mut(id_dst).push_child_tree(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}
