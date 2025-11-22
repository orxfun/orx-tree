// # EXAMPLE DEFINITION
//
// cargo run --example mutable_recursive_traversal
//
// This example demonstrates a use case where value of a node is defined
// as a function of the values of its children. Since the value of a child
// of the node also depends on values of its own children, it follows that
// the value of a node is a function of values of all of its descendants.
//
// The task is to compute and set all values of a tree given the values of
// the leaves.
//
// This is a interesting and common case in terms of requiring mutable
// recursive traversal over the tree that can be handled with different
// approaches. Some of these are demonstrated in this example.

use orx_tree::*;
use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Input(usize),
    Add,
    AddI { val: f32 },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(x) => write!(f, "Input({x})"),
            Self::Add => write!(f, "Add"),
            Self::AddI { val } => write!(f, "AddI({val})"),
        }
    }
}

#[derive(Debug)]
struct InstructionNode {
    instruction: Instruction,
    value: f32,
}

impl InstructionNode {
    fn new(instruction: Instruction, value: f32) -> Self {
        Self { instruction, value }
    }
}

impl Display for InstructionNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.instruction {
            Instruction::Input(x) => write!(f, "Input({}) => {}", x, self.value),
            Instruction::Add => write!(f, "Add => {}", self.value),
            Instruction::AddI { val } => write!(f, "AddI({}) => {}", val, self.value),
        }
    }
}

#[derive(Debug)]
struct Instructions {
    tree: DynTree<InstructionNode>,
}

impl Instructions {
    fn example() -> Self {
        let mut tree = DynTree::new(InstructionNode::new(Instruction::AddI { val: 100.0 }, 0.0));

        let mut n0 = tree.root_mut();
        let [n1, n2] = n0.push_children([
            InstructionNode::new(Instruction::Input(1), 0.0),
            InstructionNode::new(Instruction::AddI { val: 2.0 }, 0.0),
        ]);
        let _n3 = tree
            .node_mut(n1)
            .push_child(InstructionNode::new(Instruction::Input(0), 0.0));
        let [_n4, _n5] = tree.node_mut(n2).push_children([
            InstructionNode::new(Instruction::Add, 0.0),
            InstructionNode::new(Instruction::AddI { val: 5.0 }, 0.0),
        ]);

        Self { tree }
    }
}

/// Demonstrates manual mutable and recursive traversal over the tree.
///
/// Notice that we can freely walk the tree while always having a single
/// mutable reference to one node. This satisfies the borrow checker rules
/// and further allows for calling the function recursively.
///
/// Note also that, although it is not necessary in this scenario, we are
/// free to change the shape of the tree during our walk by adding nodes,
/// moving around or pruning subtrees, etc. In other words, it enables the
/// greatest freedom while it requires us to make sure that we do not have
/// errors, such as out-of-bounds errors with the `into_child_mut` call.
///
/// * Pros
///   * Complete freedom to mutate the nodes and the tree structure during
///     the walk.
///   * No intermediate allocation is required; borrow checker rules are
///     satisfied without the need to collect indices.
/// * Cons
///   * Implementor is required to define the walk. This example demonstrates
///     a depth-first walk due to the recursive calls, which is straightforward
///     to implement.
///   * Due to lack of tail-call optimization in rust, this function is likely
///     to encounter stack overflow for very deep trees.
fn recursive_traversal_over_nodes<'a>(
    inputs: &[f32],
    mut node: NodeMut<'a, Dyn<InstructionNode>>,
) -> (NodeMut<'a, Dyn<InstructionNode>>, f32) {
    let num_children = node.num_children();

    let mut children_sum = 0.0;
    for i in 0..num_children {
        let child = node.into_child_mut(i).unwrap();
        let (child, child_value) = recursive_traversal_over_nodes(inputs, child);
        children_sum += child_value;
        node = child.into_parent_mut().unwrap();
    }

    let new_value = match node.data().instruction {
        Instruction::Input(i) => inputs[i],
        Instruction::Add => children_sum,
        Instruction::AddI { val } => val + children_sum,
    };

    node.data_mut().value = new_value;

    (node, new_value)
}

/// Demonstrates recursive mutable traversal by internally collecting and storing
/// the child node indices.
///
/// This simplifies the borrow relations and allows for the recursive calls only
/// having a single mutable reference to the tree; however, each recursive call
/// requires an internal allocation.
///
/// * Pros
///   * Complete freedom to mutate the nodes and the tree structure during
///     the walk.
/// * Cons
///   * Requires to collect indices and results into an internal vector for each
///     recursive call, requiring additional allocation.
///   * Implementor is required to define the walk. This example demonstrates
///     a depth-first walk due to the recursive calls, which is straightforward
///     to implement.
///   * Due to lack of tail-call optimization in rust, this function is likely
///     to encounter stack overflow for very deep trees.
fn recursive_traversal_over_indices(
    tree: &mut DynTree<InstructionNode>,
    inputs: &[f32],
    node_idx: NodeIdx<Dyn<InstructionNode>>,
) -> f32 {
    let node = tree.node(node_idx);

    let children_ids: Vec<_> = node.children().map(|child| child.idx()).collect();
    let children: Vec<_> = children_ids
        .into_iter()
        .map(|node| recursive_traversal_over_indices(tree, inputs, node))
        .collect();

    let mut node = tree.node_mut(node_idx);

    let new_value = match node.data().instruction {
        Instruction::Input(i) => inputs[i],
        Instruction::Add => children.into_iter().sum(),
        Instruction::AddI { val } => children.into_iter().sum::<f32>() + val,
    };
    node.data_mut().value = new_value;

    new_value
}

/// Demonstrates the use of [`recursive_set`] method:
///
/// *Recursively sets the data of all nodes belonging to the subtree rooted
/// at this node using the compute_data function.*
///
/// This function fits perfectly to this and similar scenarios where we want
/// to compute values of all nodes of a tree such that the value of a node
/// depends on the values of all of its descendants, and hence the name
/// *recursive*.
///
/// * Pros
///   * More expressive in the sense that the implementor only defines how the
///     value of a node should be computed given its prior value and values of
///     its children. Iteration is abstracted away.
///   * Despite the name, the implementation actually does not require recursive
///     function calls; and hence, can work with trees of arbitrary depth without
///     the risk of stack overflow. Instead, it internally uses the [`PostOrder`]
///     traverser.
/// * Cons
///   * It only allows to set the data of the nodes; however, does not allow for
///     structural mutations.
///
/// [`recursive_set`]: orx_tree::NodeMut::recursive_set
/// [`PostOrder`]: orx_tree::PostOrder
fn recursive_set(inputs: &[f32], mut node: NodeMut<Dyn<InstructionNode>>) {
    node.recursive_set(|node_data, children_data| {
        let instruction = node_data.instruction;
        let children_sum: f32 = children_data.iter().map(|x| x.value).sum();
        let value = match node_data.instruction {
            Instruction::Input(i) => inputs[i],
            Instruction::Add => children_sum,
            Instruction::AddI { val } => val + children_sum,
        };

        InstructionNode { instruction, value }
    });
}

fn main() {
    fn test_implementation(method: &str, f: impl FnOnce(&[f32], &mut Instructions)) {
        let inputs = [10.0, 20.0];
        let mut instructions = Instructions::example();
        println!("\n\n### {method}");
        f(&inputs, &mut instructions);
        println!("\n{}\n", &instructions.tree);
    }

    test_implementation(
        "recursive_traversal_over_indices",
        |inputs, instructions| {
            let root_idx = instructions.tree.root().idx();
            recursive_traversal_over_indices(&mut instructions.tree, inputs, root_idx);
        },
    );

    test_implementation("recursive_traversal_over_nodes", |inputs, instructions| {
        recursive_traversal_over_nodes(inputs, instructions.tree.root_mut());
    });

    test_implementation("recursive_set", |inputs, instructions| {
        recursive_set(inputs, instructions.tree.root_mut());
    });
}
