use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_iterable::{IntoCloningIterable, Iterable};
#[cfg(feature = "orx-parallel")]
use orx_parallel::ParIter;
use orx_tree::*;

fn build_tree(n: usize) -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let mut dfs = Traversal.dfs().over_nodes();
    while tree.len() < n {
        let root = tree.root();
        let x: Vec<_> = root.leaves_with(&mut dfs).map(|x| x.idx()).collect();
        for idx in x.iter() {
            let count = tree.len();
            let mut node = tree.node_mut(idx);
            let num_children = 4;
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

fn path_value<'a>(path: impl IntoIterator<Item = &'a String>) -> i64 {
    path.into_iter()
        .map(|x| x.parse::<i64>().unwrap())
        .map(|x| x % 500)
        .map(fibonacci)
        .max()
        .unwrap()
}

fn paths<T: Traverser>(tree: &DynTree<String>) -> Vec<String> {
    let root = tree.root();
    root.paths::<T>()
        .map(|x| x.into_iterable())
        .max_by_key(|x| path_value(x.iter()))
        .map(|x| x.iter().cloned().collect())
        .unwrap()
}

#[cfg(feature = "orx-parallel")]
fn paths_par<T: Traverser>(tree: &DynTree<String>) -> Vec<String> {
    let root = tree.root();
    root.paths_par::<T>()
        .map(|x| x.into_iterable())
        .max_by_key(|x| path_value(x.iter()))
        .map(|x| x.iter().cloned().collect())
        .unwrap()
}

type TRAVERSER = Dfs;

fn bench(c: &mut Criterion) {
    let treatments = vec![1_024 * 64];

    let mut group = c.benchmark_group("paths_iterator");

    for n in &treatments {
        let data = build_tree(*n);

        let expected = paths::<TRAVERSER>(&data);

        group.bench_with_input(BenchmarkId::new("NodeRef::paths::<T>()", n), n, |b, _| {
            let result = paths::<TRAVERSER>(&data);
            assert_eq!(path_value(&result), path_value(&expected));
            b.iter(|| paths::<TRAVERSER>(&data))
        });

        #[cfg(feature = "orx-parallel")]
        group.bench_with_input(
            BenchmarkId::new("NodeRef::paths_par::<T>()", n),
            n,
            |b, _| {
                let result = paths_par::<TRAVERSER>(&data);
                assert_eq!(path_value(&result), path_value(&expected));
                b.iter(|| paths_par::<TRAVERSER>(&data))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
