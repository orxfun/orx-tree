use crate::*;
use std::string::{String, ToString};
use std::vec::Vec;

pub(super) fn to_str<const N: usize>(values: [i32; N]) -> [String; N] {
    values.map(|x| x.to_string())
}

pub(super) fn get_main_tree() -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let [id1, id2] = tree.root_mut().push_children(to_str([1, 2]));
    let [_id3, _id4, _id5] = tree.node_mut(id1).push_children(to_str([3, 4, 5]));
    let [id6] = tree.node_mut(id2).push_children(to_str([6]));
    let [_id7, _id8] = tree.node_mut(id6).push_children(to_str([7, 8]));
    tree
}

pub(super) fn get_main_tree_copy() -> DynTree<i32> {
    let mut tree = DynTree::new(0);
    let [id1, id2] = tree.root_mut().push_children([1, 2]);
    let [_id3, _id4, _id5] = tree.node_mut(id1).push_children([3, 4, 5]);
    let [id6] = tree.node_mut(id2).push_children([6]);
    let [_id7, _id8] = tree.node_mut(id6).push_children([7, 8]);
    tree
}

pub(super) fn get_other_tree() -> DaryTree<4, String> {
    let mut tree = DaryTree::new(10.to_string());
    let [id11, _id12] = tree.root_mut().push_children(to_str([11, 12]));
    let [_id13, _id14] = tree.node_mut(id11).push_children(to_str([13, 14]));
    tree
}

pub(super) fn get_other_tree_copy() -> DaryTree<4, i32> {
    let mut tree = DaryTree::new(10);
    let [id11, _id12] = tree.root_mut().push_children([11, 12]);
    let [_id13, _id14] = tree.node_mut(id11).push_children([13, 14]);
    tree
}

pub(super) fn collect_sorted_subtree<V>(node: Node<V>) -> Vec<V::Item>
where
    V: TreeVariant,
    V::Item: Clone + Ord,
{
    let mut values: Vec<_> = node.walk::<Bfs>().cloned().collect();
    values.sort();
    values
}
