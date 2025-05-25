// cargo run --release --features orx-parallel --example demo_parallelization

use orx_tree::*;

fn build_tree(total_depth: usize, num_children: usize) -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let mut dfs = Traversal.dfs().over_nodes();

    for _ in 0..total_depth {
        let root = tree.root();
        let leaves: Vec<_> = root.leaves_with(&mut dfs).map(|x| x.idx()).collect();
        for idx in leaves {
            let count = tree.len();
            let mut node = tree.node_mut(&idx);
            for j in 0..num_children {
                node.push_child((count + j).to_string());
            }
        }
    }

    tree
}

fn main() {
    let input = build_tree(10, 4);
    let expected_num_characters = 8675597;

    // computation using iterators

    let total_num_characters: usize = input.iter().map(|x| x.len()).sum();
    assert_eq!(total_num_characters, expected_num_characters);

    #[cfg(feature = "orx-parallel")]
    {
        // computation using parallel iterator: replace `iter()` with `par()`

        let total_num_characters: usize = input.par().map(|x| x.len()).sum();
        assert_eq!(total_num_characters, expected_num_characters);

        // configure parallel computation
        let total_num_characters: usize = input
            .par()
            .num_threads(2)
            .chunk_size(64)
            .map(|x| x.len())
            .sum();
        assert_eq!(total_num_characters, expected_num_characters);

        // consuming parallel iterator: replace `into_iter` with `into_par`
        let total_num_characters: usize = input.into_par().map(|x| x.len()).sum();
        assert_eq!(total_num_characters, expected_num_characters);
    }
}
