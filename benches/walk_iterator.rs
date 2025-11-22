use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
#[cfg(feature = "parallel")]
use orx_parallel::*;
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
            let num_children = 10;
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

fn arbitrary_order_iter(tree: &DynTree<String>) -> i64 {
    tree.iter()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

#[cfg(feature = "parallel")]
fn arbitrary_order_par_iter(tree: &DynTree<String>) -> i64 {
    tree.par()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn arbitrary_order_par_iter_with_rayon(tree: &DynTree<String>) -> i64 {
    tree.iter()
        .par_bridge()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn walk<T: Traverser>(tree: &DynTree<String>) -> i64 {
    tree.root()
        .walk::<T>()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

#[cfg(feature = "parallel")]
fn walk_par<T: Traverser>(tree: &DynTree<String>) -> i64 {
    tree.root()
        .walk_par::<T>()
        .map(|x| x.parse::<usize>().unwrap())
        .map(|x| fibonacci(x as i64 % 500))
        .sum()
}

fn bench(c: &mut Criterion) {
    let treatments = vec![1_024 * 64];

    let mut group = c.benchmark_group("walk_iterator");

    for n in &treatments {
        let data = build_tree(*n);

        let expected = arbitrary_order_iter(&data);

        group.bench_with_input(BenchmarkId::new("arbitrary_order_iter", n), n, |b, _| {
            let result = arbitrary_order_iter(&data);
            assert_eq!(result, expected);
            b.iter(|| arbitrary_order_iter(&data))
        });

        #[cfg(feature = "parallel")]
        group.bench_with_input(
            BenchmarkId::new("arbitrary_order_par_iter", n),
            n,
            |b, _| {
                let result = arbitrary_order_par_iter(&data);
                assert_eq!(result, expected);
                b.iter(|| arbitrary_order_par_iter(&data))
            },
        );

        group.bench_with_input(
            BenchmarkId::new("arbitrary_order_par_iter_with_rayon", n),
            n,
            |b, _| {
                let result = arbitrary_order_par_iter_with_rayon(&data);
                assert_eq!(result, expected);
                b.iter(|| arbitrary_order_par_iter_with_rayon(&data))
            },
        );

        group.bench_with_input(BenchmarkId::new("walk::<Dfs>", n), n, |b, _| {
            let result = walk::<Dfs>(&data);
            assert_eq!(result, expected);
            b.iter(|| walk::<Dfs>(&data))
        });

        #[cfg(feature = "parallel")]
        group.bench_with_input(BenchmarkId::new("walk_par::<Dfs>", n), n, |b, _| {
            let result = walk_par::<Dfs>(&data);
            assert_eq!(result, expected);
            b.iter(|| walk_par::<Dfs>(&data))
        });

        group.bench_with_input(BenchmarkId::new("walk::<Bfs>", n), n, |b, _| {
            let result = walk::<Bfs>(&data);
            assert_eq!(result, expected);
            b.iter(|| walk::<Bfs>(&data))
        });

        #[cfg(feature = "parallel")]
        group.bench_with_input(BenchmarkId::new("walk_par::<Bfs>", n), n, |b, _| {
            let result = walk_par::<Bfs>(&data);
            assert_eq!(result, expected);
            b.iter(|| walk_par::<Bfs>(&data))
        });

        group.bench_with_input(BenchmarkId::new("walk::<PostOrder>", n), n, |b, _| {
            let result = walk::<PostOrder>(&data);
            assert_eq!(result, expected);
            b.iter(|| walk::<PostOrder>(&data))
        });

        #[cfg(feature = "parallel")]
        group.bench_with_input(BenchmarkId::new("walk_par::<PostOrder>", n), n, |b, _| {
            let result = walk_par::<PostOrder>(&data);
            assert_eq!(result, expected);
            b.iter(|| walk_par::<PostOrder>(&data))
        });
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
