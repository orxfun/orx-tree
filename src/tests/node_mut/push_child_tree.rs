use crate::tests::node_mut::utils::{
    collect_sorted_subtree, get_main_tree, get_main_tree_copy, get_other_tree, get_other_tree_copy,
    to_str,
};
use crate::*;
use std::string::ToString;
use std::vec::Vec;

#[test]
fn push_child_tree_cloned() {
    let tree = get_main_tree();
    let initial_nodes = collect_sorted_subtree(tree.root());

    let other = get_other_tree();
    let ids_other: Vec<_> = other.root().indices::<Bfs>().collect();

    for i in 0..tree.len() {
        for id_src in ids_other.iter().copied() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(collect_sorted_subtree(other.node(id_src)));
            expected_nodes.sort();

            let subtree = other.node(id_src).as_cloned_subtree();
            tree.node_mut(id_dst).push_child_tree(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}

#[test]
fn push_child_tree_copied() {
    let tree = get_main_tree_copy();
    let initial_nodes = collect_sorted_subtree(tree.root());

    let other = get_other_tree_copy();
    let ids_other: Vec<_> = other.root().indices::<Bfs>().collect();

    for i in 0..tree.len() {
        for id_src in ids_other.iter().copied() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(collect_sorted_subtree(other.node(id_src)));
            expected_nodes.sort();

            let subtree = other.node(id_src).as_copied_subtree();
            tree.node_mut(id_dst).push_child_tree(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}

#[test]
fn push_child_tree_moved() {
    let tree = get_main_tree();
    let initial_nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
    let other = get_other_tree();

    for i in 0..tree.len() {
        for j in 0..other.len() {
            let mut tree = tree.clone();
            let mut other = other.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_src = other.root().indices::<Bfs>().nth(j).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(collect_sorted_subtree(other.node(id_src)));
            expected_nodes.sort();

            let subtree = other.node_mut(id_src).into_subtree();
            tree.node_mut(id_dst).push_child_tree(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}

#[test]
fn push_child_tree_within_cloned() {
    let tree = get_main_tree();

    let initial_nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();

    for i in 0..tree.len() {
        for j in 0..tree.len() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_src = tree.root().indices::<Bfs>().nth(j).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(collect_sorted_subtree(tree.node(id_src)));
            expected_nodes.sort();

            let subtree = tree.node(id_src).as_cloned_subtree_within();
            tree.node_mut(id_dst).push_child_tree_within(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().cloned().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}

#[test]
fn push_child_tree_within_copied() {
    let tree = get_main_tree_copy();

    let initial_nodes = collect_sorted_subtree(tree.root());

    for i in 0..tree.len() {
        for j in 0..tree.len() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_src = tree.root().indices::<Bfs>().nth(j).unwrap();

            let mut expected_nodes = initial_nodes.clone();
            expected_nodes.extend(collect_sorted_subtree(tree.node(id_src)));
            expected_nodes.sort();

            let subtree = tree.node(id_src).as_cloned_subtree_within();
            tree.node_mut(id_dst).push_child_tree_within(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
            nodes.sort();

            assert_eq!(nodes, expected_nodes);
        }
    }
}

#[test]
fn push_child_tree_within_moved() {
    let tree = get_main_tree_copy();

    let mut initial_nodes: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    initial_nodes.sort();

    for i in 0..tree.len() {
        for j in 0..tree.len() {
            let mut tree = tree.clone();
            let id_dst = tree.root().indices::<Bfs>().nth(i).unwrap();
            let id_src = tree.root().indices::<Bfs>().nth(j).unwrap();

            let node_src = tree.node(id_src);
            if id_src == id_dst || node_src.is_ancestor_of(id_dst) {
                continue;
            }

            let subtree = id_src.into_subtree_within();
            tree.node_mut(id_dst).push_child_tree_within(subtree);

            let mut nodes: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
            nodes.sort();

            assert_eq!(nodes, initial_nodes);
        }
    }
}

#[test]
#[should_panic]
fn push_child_tree_within_moved_to_itself() {
    let mut tree = DynTree::new(0.to_string());
    let [id1, _] = tree.root_mut().push_children(to_str([1, 2]));
    let [_id3, _id4, _id5] = tree.node_mut(id1).push_children(to_str([3, 4, 5]));

    let id_dst = id1;
    let id_src = id1;

    let subtree = id_src.into_subtree_within();
    tree.node_mut(id_dst).push_child_tree_within(subtree);
}

#[test]
#[should_panic]
fn push_child_tree_within_moved_to_its_descendant() {
    let mut tree = DynTree::new(0.to_string());
    let [id1, _] = tree.root_mut().push_children(to_str([1, 2]));
    let [_id3, id4, _id5] = tree.node_mut(id1).push_children(to_str([3, 4, 5]));

    let id_dst = id4;
    let id_src = id1;

    let subtree = id_src.into_subtree_within();
    tree.node_mut(id_dst).push_child_tree_within(subtree);
}
