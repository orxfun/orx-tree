use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
#[cfg(feature = "parallel")]
use orx_parallel::ParIter;
use orx_tree::*;

fn build_tree(n: usize) -> DynTree<String> {
    let mut tree = DynTree::new(0.to_string());
    let mut dfs = Traversal.dfs().over_nodes();
    while tree.len() < n {
        let root = tree.root();
        let x: Vec<_> = root.leaves_with(&mut dfs).map(|x| x.idx()).collect();
        for idx in x.iter().copied() {
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

fn fibonacci(n: u64) -> u64 {
    let mut a = 0;
    let mut b = 1;
    for _ in 0..n {
        let c = a + b;
        a = b;
        b = c;
    }
    a
}

fn compute_subtree_value(subtree: &DynNode<String>) -> u64 {
    subtree
        .walk::<Dfs>()
        .map(|x| x.parse::<u64>().unwrap())
        .map(|x| fibonacci(x % 1000))
        .sum()
}

fn children(tree: &DynTree<String>) -> u64 {
    tree.root()
        .children()
        .map(|x| compute_subtree_value(&x))
        .sum()
}

#[cfg(feature = "parallel")]
fn children_par(tree: &DynTree<String>) -> u64 {
    tree.root()
        .children_par()
        .map(|x| compute_subtree_value(&x))
        .sum()
}

#[cfg(feature = "parallel")]
fn children_par_2threads(tree: &DynTree<String>) -> u64 {
    tree.root()
        .children_par()
        .num_threads(2)
        .map(|x| compute_subtree_value(&x))
        .sum()
}

fn bench(c: &mut Criterion) {
    let treatments = vec![1_024 * 64];

    let mut group = c.benchmark_group("children_iterator");

    for n in &treatments {
        let tree = build_tree(*n);

        let expected = children(&tree);

        group.bench_with_input(BenchmarkId::new("NodeRef::children()", n), n, |b, _| {
            let result = children(&tree);
            assert_eq!(result, expected);
            b.iter(|| children(&tree))
        });

        #[cfg(feature = "parallel")]
        group.bench_with_input(
            BenchmarkId::new("NNodeRef::children_par()", n),
            n,
            |b, _| {
                let result = children_par(&tree);
                assert_eq!(result, expected);
                b.iter(|| children_par(&tree))
            },
        );

        #[cfg(feature = "parallel")]
        group.bench_with_input(
            BenchmarkId::new("NodeRef::children_par().num_threads(2)", n),
            n,
            |b, _| {
                let result = children_par_2threads(&tree);
                assert_eq!(result, expected);
                b.iter(|| children_par_2threads(&tree))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
