use orx_tree::*;

struct Inputs {/* placeholder: useful immutable inputs */}

/// Returns the tree below and index to node 3.
///
/// ```
///       1
///     ╱ ╲
///    ╱   ╲
///   2     3
///  ╱ ╲   ╱ ╲
/// 4   5 6   7
/// |     |  ╱ ╲
/// 8     9 10  11
/// ```
fn init_tree() -> (Tree<Dyn<u64>>, NodeIdx<Dyn<u64>>) {
    let mut tree = DynTree::new(1u64);

    let mut root = tree.root_mut();
    let [id2, id3] = root.push_children([2, 3]);
    let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
    let _id8 = tree.node_mut(&id4).push_child(8);
    let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
    let _id9 = tree.node_mut(&id6).push_child(9);
    tree.node_mut(&id7).push_children([10, 11]);

    (tree, id3)
}

/// Placeholder: counts the number of all children as an example
fn execute_rec(_inputs: &Inputs, node: &Node<'_, Dyn<u64>>) -> u64 {
    node.walk::<Bfs>().count() as u64 - 1
}

fn implementation1() {
    let inputs = Inputs {};
    let (mut tree, node_idx) = init_tree();

    let node = tree.node(&node_idx);
    let children_ids = node.children().map(|child| child.idx()).collect::<Vec<_>>();

    let mut new_children = vec![];
    for node_id in children_ids {
        let node = tree.node(&node_id);
        let value = execute_rec(&inputs, &node);
        new_children.push(value);
    }

    let mut node = tree.node_mut(&node_idx);
    for child in new_children { /* use children and node here */ }
}

fn implementation2() {
    let inputs = Inputs {};
    let (mut tree, node_idx) = init_tree();

    let node = tree.node_mut(&node_idx);

    let mut new_children = vec![];
    for child in node.children() {
        let value = execute_rec(&inputs, &child);
        new_children.push(value);
    }

    for child in new_children { /* use children and node here */ }
}

fn main() {}
