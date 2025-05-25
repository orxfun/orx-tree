use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
#[cfg(feature = "orx-parallel")]
use orx_parallel::ParIter;
use orx_tree::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

fn build_tree(n: usize) -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let mut dfs = Traversal.dfs().over_nodes();
    while tree.len() < n {
        let root = tree.root();
        let x: Vec<_> = root.leaves_with(&mut dfs).map(|x| x.idx()).collect();
        for idx in x.iter() {
            let count = tree.len();
            let mut node = tree.node_mut(idx);
            let num_children = 20;
            for j in 0..num_children {
                node.push_child((count + j).to_string());
            }
        }
    }
    tree
}

fn fibonacci(n: i64) -> i64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn tree_iter(tree: &DynTree<String>) -> i64 {
    tree.iter()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_dfs(tree: &DynTree<String>) -> i64 {
    tree.root()
        .walk::<Dfs>()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_bfs(tree: &DynTree<String>) -> i64 {
    tree.root()
        .walk::<Bfs>()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_par_x(tree: &DynTree<String>) -> i64 {
    tree.par()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_iter_rayon(tree: &DynTree<String>) -> i64 {
    tree.iter()
        .par_bridge()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn bench(c: &mut Criterion) {
    let treatments = vec![1_024 * 64];

    let mut group = c.benchmark_group("parallelization_ref");

    for n in &treatments {
        let data = build_tree(*n);

        let expected = tree_iter(&data);

        group.bench_with_input(BenchmarkId::new("Tree::iter()", n), n, |b, _| {
            let result = tree_iter(&data);
            assert_eq!(result, expected);
            b.iter(|| tree_iter(&data))
        });

        group.bench_with_input(
            BenchmarkId::new("Tree::root().walk::<Dfs>()", n),
            n,
            |b, _| {
                let result = tree_dfs(&data);
                assert_eq!(result, expected);
                b.iter(|| tree_dfs(&data))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Tree::root().walk::<Bfs>()", n),
            n,
            |b, _| {
                let result = tree_bfs(&data);
                assert_eq!(result, expected);
                b.iter(|| tree_bfs(&data))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Tree::par_x() - orx-parallel", n),
            n,
            |b, _| {
                let result = tree_par_x(&data);
                assert_eq!(result, expected);
                b.iter(|| tree_par_x(&data))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Tree::iter().par_bridge() - rayon", n),
            n,
            |b, _| {
                let result = tree_iter_rayon(&data);
                assert_eq!(result, expected);
                b.iter(|| tree_iter_rayon(&data))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
