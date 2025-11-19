// cargo run --release --features parallel --example bench_parallelization
// cargo run --release --features parallel --example bench_parallelization -- --help
// cargo run --release --features parallel --example bench_parallelization -- --len 50000 --num-repetitions 20

mod utils;

use clap::Parser;
use orx_tree::*;
use rayon::iter::{ParallelBridge, ParallelIterator};
use utils::timed_collect_all;

#[derive(Parser, Debug)]
struct Args {
    /// Number of items in the input iterator.
    #[arg(long, default_value_t = 1_000_000)]
    len: usize,
    /// Number of repetitions to measure time; total time will be reported.
    #[arg(long, default_value_t = 100)]
    num_repetitions: usize,
}

fn fibonacci(n: usize) -> usize {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn build_tree(total_depth: usize) -> DaryTree<4, usize> {
    let mut tree = DaryTree::new(0);
    let mut dfs = Traversal.dfs().over_nodes();

    for _ in 0..total_depth {
        let root = tree.root();
        let leaves: Vec<_> = root.leaves_with(&mut dfs).map(|x| x.idx()).collect();
        for idx in leaves {
            let count = tree.len();
            let mut node = tree.node_mut(&idx);
            for j in 0..4 {
                node.push_child(count + j);
            }
        }
    }

    tree
}

fn main() {
    let args = Args::parse();

    let mut expected_output = {
        let tree = build_tree(10);

        tree.iter()
            .filter(|x| *x % 3 != 0)
            .map(|x| x + fibonacci(x % 1000))
            .filter(|x| x % 2 == 0)
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
    };
    expected_output.sort();

    let computations: Vec<utils::ComputeTuple<Vec<String>>> = vec![
        #[cfg(feature = "parallel")]
        (
            "Sequential computation over Tree",
            Box::new(move || {
                let tree = build_tree(10);

                tree.iter()
                    .filter(|x| *x % 3 != 0)
                    .map(|x| x + fibonacci(x % 1000))
                    .filter(|x| x % 2 == 0)
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            }),
        ),
        #[cfg(feature = "parallel")]
        (
            "Parallelized over Tree using parallel",
            Box::new(move || {
                let tree = build_tree(10);

                tree.par() // replace iter (into_iter) with par (into_par) to parallelize !
                    .filter(|x| *x % 3 != 0)
                    .map(|x| x + fibonacci(x % 1000))
                    .filter(|x| x % 2 == 0)
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            }),
        ),
        (
            "Parallelized over Tree using rayon's par-bridge",
            Box::new(move || {
                let tree = build_tree(10);

                tree.iter()
                    .par_bridge()
                    .filter(|x| *x % 3 != 0)
                    .map(|x| x + fibonacci(x % 1000))
                    .filter(|x| x % 2 == 0)
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
            }),
        ),
    ];

    timed_collect_all(
        "benchmark_parallelization",
        args.num_repetitions,
        &expected_output,
        &computations,
    );
}
