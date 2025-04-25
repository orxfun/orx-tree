use orx_tree::*;
use std::fmt::Display;

const INPUTS_COUNT: usize = 2;

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Input(usize),
    Add,
    AddI { val: f32 },
}

impl Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Input(x) => write!(f, "Input({})", x),
            Self::Add => write!(f, "Add"),
            Self::AddI { val } => write!(f, "AddI({})", val),
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
struct MyTree {
    tree: DynTree<InstructionNode>,
}

impl MyTree {
    fn example() -> Self {
        let mut tree = DynTree::new(InstructionNode::new(Instruction::AddI { val: 100.0 }, 0.0));

        let mut n0 = tree.root_mut();
        let [n1, n2] = n0.push_children([
            InstructionNode::new(Instruction::Input(1), 0.0),
            InstructionNode::new(Instruction::AddI { val: 2.0 }, 0.0),
        ]);
        let _n3 = tree
            .node_mut(&n1)
            .push_child(InstructionNode::new(Instruction::Input(0), 0.0));
        let [_n4, _n5] = tree.node_mut(&n2).push_children([
            InstructionNode::new(Instruction::Add, 0.0),
            InstructionNode::new(Instruction::AddI { val: 5.0 }, 0.0),
        ]);

        Self { tree }
    }

    fn execute_rec(&mut self, inputs: &[f32], node_idx: NodeIdx<Dyn<InstructionNode>>) -> f32 {
        let node = self.tree.node(&node_idx);

        let children_ids = node.children().map(|child| child.idx()).collect::<Vec<_>>();
        let mut children = vec![];

        for node in children_ids {
            let value = self.execute_rec(inputs, node);
            children.push(value);
        }

        let mut node = self.tree.node_mut(&node_idx);

        let new_value = match node.data().instruction {
            Instruction::Input(i) => inputs[i],
            Instruction::Add => children.into_iter().sum(),
            Instruction::AddI { val } => children.into_iter().sum::<f32>() + val,
        };
        (*node.data_mut()).value = new_value;

        new_value
    }
}

fn execute<'a>(
    inputs: &[f32],
    mut node: NodeMut<'a, Dyn<InstructionNode>>,
) -> (NodeMut<'a, Dyn<InstructionNode>>, f32) {
    let num_children = node.num_children();

    let mut children_sum = 0.0;
    for i in 0..num_children {
        let child = node.into_child_mut(i).unwrap();
        let (child, child_value) = execute(inputs, child);
        children_sum += child_value;
        node = child.into_parent_mut().unwrap();
    }

    let new_value = match node.data().instruction {
        Instruction::Input(i) => inputs[i],
        Instruction::Add => children_sum,
        Instruction::AddI { val } => val + children_sum,
    };

    (*node.data_mut()).value = new_value;

    (node, new_value)
}

fn test_implementation(method: &str, f: impl FnOnce(&[f32], &mut MyTree)) {
    let inputs = [10.0, 20.0];

    let mut tree = MyTree::example();

    println!("\n\n# {}", method);
    println!("\ninputs = {:?}\n", &inputs);
    println!("Before execute:\n{}\n", &tree.tree);
    f(&inputs, &mut tree);
    println!("After execute:\n{}\n", &tree.tree);
}

fn main() {
    test_implementation("IMPL OVER CHILDREN INDICES", |inputs, tree| {
        tree.execute_rec(inputs, tree.tree.root().idx());
    });

    test_implementation(
        "IMPL WITH INTO_CHILD_MUT & INTO_PARENT_MUT",
        |inputs, tree| {
            execute(&inputs, tree.tree.root_mut());
        },
    );

    test_implementation("IMPL recursive_set", |inputs, tree| {
        tree.tree
            .root_mut()
            .recursive_set(|node_data, children_data| {
                let instruction = node_data.instruction;
                let children_sum: f32 = children_data.iter().map(|x| x.value).sum();
                let value = match node_data.instruction {
                    Instruction::Input(i) => inputs[i],
                    Instruction::Add => children_sum,
                    Instruction::AddI { val } => val + children_sum,
                };

                InstructionNode { instruction, value }
            });
    });
}
