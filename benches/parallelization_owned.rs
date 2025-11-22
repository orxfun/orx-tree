use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
#[cfg(feature = "parallel")]
use orx_parallel::ParIter;
use orx_tree::*;
use rayon::iter::{ParallelBridge, ParallelIterator};

fn build_tree(n: usize) -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let mut dfs = Traversal.dfs().over_nodes();
    while tree.len() < n {
        let root = tree.root();
        let x: Vec<_> = root.leaves_with(&mut dfs).map(|x| x.idx()).collect();
        for idx in x.iter().copied() {
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

fn tree_into_iter(tree: DynTree<String>) -> i64 {
    tree.into_iter()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_into_dfs(mut tree: DynTree<String>) -> i64 {
    tree.root_mut()
        .into_walk::<Dfs>()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_into_bfs(mut tree: DynTree<String>) -> i64 {
    tree.root_mut()
        .into_walk::<Bfs>()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

#[cfg(feature = "parallel")]
fn tree_into_par_x(tree: DynTree<String>) -> i64 {
    tree.into_par()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn tree_into_iter_rayon(tree: DynTree<String>) -> i64 {
    tree.into_iter()
        .par_bridge()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn bench(c: &mut Criterion) {
    let treatments = vec![1_024 * 64];

    let mut group = c.benchmark_group("parallelization_owned");

    for n in &treatments {
        let data = build_tree(*n);

        let expected = tree_into_iter(data.clone());

        group.bench_with_input(BenchmarkId::new("Tree::into_iter()", n), n, |b, _| {
            let result = tree_into_iter(data.clone());
            assert_eq!(result, expected);
            b.iter(|| tree_into_iter(data.clone()))
        });

        group.bench_with_input(
            BenchmarkId::new("Tree::root().into_walk::<Dfs>()", n),
            n,
            |b, _| {
                let result = tree_into_dfs(data.clone());
                assert_eq!(result, expected);
                b.iter(|| tree_into_dfs(data.clone()))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Tree::root().into_walk::<Bfs>()", n),
            n,
            |b, _| {
                let result = tree_into_bfs(data.clone());
                assert_eq!(result, expected);
                b.iter(|| tree_into_bfs(data.clone()))
            },
        );

        #[cfg(feature = "parallel")]
        group.bench_with_input(
            BenchmarkId::new("Tree::into_par_x() - parallel", n),
            n,
            |b, _| {
                let result = tree_into_par_x(data.clone());
                assert_eq!(result, expected);
                b.iter(|| tree_into_par_x(data.clone()))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("Tree::into_iter().par_bridge() - rayon", n),
            n,
            |b, _| {
                let result = tree_into_iter_rayon(data.clone());
                assert_eq!(result, expected);
                b.iter(|| tree_into_iter_rayon(data.clone()))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
