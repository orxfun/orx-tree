# orx-tree

[![orx-tree crate](https://img.shields.io/crates/v/orx-tree.svg)](https://crates.io/crates/orx-tree)
[![orx-tree crate](https://img.shields.io/crates/d/orx-tree.svg)](https://crates.io/crates/orx-tree)
[![orx-tree documentation](https://docs.rs/orx-tree/badge.svg)](https://docs.rs/orx-tree)

A beautiful tree 🌳 with convenient, efficient, parallelizable growth, mutation and traversal features.

## Tree Variants

[`Tree`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html) is generic over variants that define the way the children are stored:

* [`DynTree<T>`](https://docs.rs/orx-tree/latest/orx_tree/type.DynTree.html), or equivalently **Tree&lt;Dyn&lt;T&gt;&gt;**, is a tree where each node may contain references to any number of children stored as a vector.
* [`DaryTree<D, T>`](https://docs.rs/orx-tree/latest/orx_tree/type.DaryTree.html), or equivalently **Tree&lt;Dary&lt;D, T&gt;&gt;**, is a tree where each node may contain at most **D** child references stored inlined as an array.
* [`BinaryTree<T>`](https://docs.rs/orx-tree/latest/orx_tree/type.BinaryTree.html) is simply a shorthand for **DaryTree&lt;2, T&gt;**.

## Recursive Nature of Trees

Note that [`Tree`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html) has only few methods which mainly allow access to the root or to any node using node indices. Since every node is the root of a subtree, the core tree functionalities are provided as methods of [`NodeRef`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html) and [`NodeMut`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html), which are immutable and mutable nodes, respectively.

## Iterations

### Walks over the Tree

We can visit all nodes of the tree in various ways. The way we *walk*, or the order of visited nodes, is determined by a generic traversal parameter.

To demonstrate, consider the following methods of a tree node:

* [`walk::<Bfs>()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.walk) creates an iterator that visits all the nodes belonging to the subtree rooted at the *node* in the breadth-first order.
* [`walk_mut::<Dfs>()`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.walk_mut) creates a mutable iterator, this time in depth-first order.
* [`into_walk::<PostOrder>()`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.into_walk), on the other hand, takes the subtree rooted at the *node* out of the tree and yields the elements in post-order.

Walk iterators might yield node values or nodes with access to children, parent, siblings, etc. Further, node depth and/or its position among its siblings might be added. These more specialized traversals can be created conveniently using the [`Traversal`](https://docs.rs/orx-tree/latest/orx_tree/traversal/struct.Traversal.html) builder type.

*You may see the [walks](https://github.com/orxfun/orx-tree/blob/main/examples/walks.rs) example that demonstrates different ways to walk the tree with traversal variants (`cargo run --example walks`).*

```rust
use orx_tree::*;

//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱     ╱ ╲
// 4     5   6
let mut tree = DynTree::new(1);
let [id2, id3] = tree.root_mut().push_children([2, 3]);
let id4 = tree.node_mut(&id2).push_child(4);
tree.node_mut(&id3).push_children([5, 6]);

let root = tree.root();
assert_eq!(root.walk::<Dfs>().copied().collect::<Vec<_>>(), [1, 2, 4, 3, 5, 6]);
assert_eq!(root.walk::<Bfs>().copied().collect::<Vec<_>>(), [1, 2, 3, 4, 5, 6]);
assert_eq!(root.walk::<PostOrder>().copied().collect::<Vec<_>>(), [4, 2, 5, 6, 3, 1]);

// create a re-usable BFS traverser, with additional access to depth and sibling-idx
let mut t = Traversal.bfs().with_depth().with_sibling_idx();
assert_eq!(
    root.walk_with(&mut t).collect::<Vec<_>>(),
     [(0, 0, &1), (1, 0, &2), (1, 1, &3), (2, 0, &4), (2, 0, &5), (2, 1, &6)]
);
```

### Custom Walk

In addition to common traversal strategies, we can create a **custom iterator** by simply calling [`custom_walk(next_node)`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.custom_walk) where the argument `next_node` is a function with signature `Fn(Node) -> Option<Node>` defining the traversal strategy.

### Special Iterators

In addition to walks over subtrees, the following iterators are useful in special use cases.

* [`leaves::<Bfs>()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.leaves) returns leaf nodes in breadth-first order.
* [`paths::<Dfs>()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.paths) returns all the paths or sequences of nodes connecting the *node* to all of its leaves in the depth-first order.
* [`ancestors()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.ancestors) provides an upward iterator from the *node* to the root of the tree.

*You may see the [special iterators](https://github.com/orxfun/orx-tree/blob/main/examples/special_iterators.rs) example.*

### Manual Traversals

Alternatively, we can move on nodes of the tree freely:

* **↓** [`child(child_idx)`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.child), [`children()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.children), [`children_mut()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.children_mut), [`into_child_mut(child_idx)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.into_child_mut)
* **↑** [`parent()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.parent), [`into_parent_mut()`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.into_parent_mut), etc.

*You may see [manual iteration](https://github.com/orxfun/orx-tree/blob/main/examples/manual_iteration.rs) and [mutable_recursive_traversal](https://github.com/orxfun/orx-tree/blob/main/examples/mutable_recursive_traversal.rs) examples*

### Arbitrary Order Iterators

The tree naturally implements `IntoIterator`, [`Collection`](https://docs.rs/orx-iterable/latest/orx_iterable/trait.Collection.html) and [`CollectionMut`](https://docs.rs/orx-iterable/latest/orx_iterable/trait.CollectionMut.html) providing iterators via `into_iter`, `iter` and `iter_mut` methods. These iterators yield elements in an arbitrary but deterministic order.

## Parallelization

`Tree` aims to enable convenient parallel computation for all iterators, traversals or walks mentioned above using the [orx-parallel](https://crates.io/crates/orx-parallel) feature (see [features](#features) section). Parallel counterparts return a [`ParIter`](https://docs.rs/orx-parallel/latest/orx_parallel/trait.ParIter.html) rather than a sequential `Iterator`.

[`tree.par()`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html#method.par) and [`tree.into_par()`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html#method.into_par) return parallel iterators over all nodes of the tree. Examples can be found in [`demo_parallelization`](https://github.com/orxfun/orx-tree/blob/main/examples/demo_parallelization.rs) example. Importantly note that the tree defines its own concurrent iterators, and hence, allows for efficient computation, which is often not possible with generic implementations. In order to check the impact in performance, you may use the lightweight benchmark example [`bench_parallelization`](https://github.com/orxfun/orx-tree/blob/main/examples/bench_parallelization.rs):

* `Sequential computation over Tree : 18.96s`
* `Parallelized over Tree using orx-parallel : 6.02s`
* `Parallelized over Tree using rayon's par-bridge : 81.10s`

Remaining walks and traversals can be parallelized simply by adding **_par** suffix to names of their sequential counterparts:

[`children_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.children_par) | 
[`ancestors_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.ancestors_par) |
[`custom_walk_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.custom_walk_par) |
[`walk_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.walk_par) |
[`walk_into_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.walk_into_par) |
[`paths_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.paths_par) |
[`paths_with_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.paths_with_par) |
[`leaves_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.leaves_par) |
[`leaves_with_par`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.leaves_with_par) |

## Constant Time Access to Nodes via Node Indices

A [`NodeIdx`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeIdx.html) for a [`Tree`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html) is similar to `usize` for a slice in that it allows constant time access to the node it is created for.

On the other hand, it is more specific for the node due to the following:

* usize represents a position of the slice. Say we have the slice *[a, b, c]*. Currently, index 0 points to element *a*. However, if we swap the first and third elements, index 0 will now be pointing to *c* because the usize represents a position on the slice.
* A node index represents the node it is created for. If the index is created for node *a*, it will always point to this node no matter how many times we move the node in the tree. Further, we cannot use this node index on another tree and it does not allow access to another node if node *a* is removed from the tree.

Therefore, node access through node indices is safe. To demonstrate, assume we have the following command:

```rust ignore
let idx = tree.root_mut().push_child(42);
```

Here, `idx` does not have a lifetime attached to the `tree`, yet it refers to the node on this tree which currently holds value 42 (thanks to pinned element guarantees). This allows for a safe and efficient access to the nodes:

* `tree.node(&idx)` provides a constant time access to this particular node.
* `another_tree.node(&idx)` is an out-of-bounds error.
* `tree.node(&idx)` after removing the node from the tree, say by `tree.node_mut(&idx).prune()` call, is a removed-node error.

## Cache Locality

Nodes of the tree are stored in an underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) with pinned element guarantees. This allows for keeping the nodes close to each other improving cache locality while still providing with constant time mutation methods.

## Convenient Mutations

The tree aims to make every move on the tree possible, convenient and efficient.

### Growth & Move Subtrees Around

The following methods demonstrate downward growth by adding descendants to a node:

* [`push_child(value)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child) => adds a single child
* [`push_children(values)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_children) => adds a constant number of children
* [`extend_children(values)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.extend_children) => adds a variable number of children provided by an iterator
* [`push_child_tree(subtree)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child_tree) => appends the subtree as descendants of the *node* such that the root of the subtree is the child of the *node*
* [`push_child_tree_within(subtree)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child_tree_within) => similar to the above except that the subtree belongs to the same tree, we might be moving or cloning the subtree

These methods have the *sibling* variants such as [`push_sibling`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_sibling) rather than *push_child* which additionally allows to define the side of the new sibling (to the left or right).

Further, [`push_parent(value)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_parent) allows to push a node in between a node and its parent.

These methods aim to enable inserting nodes or subtrees at any position of the tree.

Note that all the growth methods return the indices of the created nodes allowing for a fluent growth of the tree.

Additionally, the tree provides methods for special moves such as [`swap_subtrees`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html#method.swap_subtrees) to swap components of the same tree.

### Removals

We can take out a node from the tree, while connecting its parent to its children via the [`take_out`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.take_out) method.

Alternatively, we can [`prune`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.prune) by removing a subtree rooted at a particular node, and receive the value of the root node of the removed subtree.

Alternatively, we can turn a mutable node into an [`into_walk`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.into_walk) iterator. Similar to *prune*, this will remove the subtree. However, we are flexible on what we do with the removed subtree:

* We can simply discard it. Then, *into_walk* behaves similar to *prune*.
* We can iterate over the removed nodes in the order of the generic traversal parameter and use the data however we need.
* Or we can attach the removed subtree at a desired position of another tree by passing it to methods such as [`push_child_tree(subtree)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child_tree).

# Features

* **orx-parallel**: Tree allows efficient parallel processing through [concurrent iterators](https://crates.io/crates/orx-concurrent-iter) and [parallel iterators](https://crates.io/crates/orx-parallel). See [parallelization section](#parallelization) for details. This feature is added as default and requires **std**. Therefore, please use `cargo add orx-tree --no-default-features` for **no-std** use cases.

* **serde**: Tree implements `Serialize` and `Deserialize` traits; the "serde" feature needs to be added when required. It uses a linearized representation of the tree as a [`DepthFirstSequence`](https://docs.rs/orx-tree/latest/orx_tree/struct.DepthFirstSequence.html). You may find de-serialization examples in the corresponding [test file](https://github.com/orxfun/orx-tree/blob/main/tests/serde.rs).

# Examples

The following example demonstrates the basic usage of the `Tree` by constructing and playing around with mutation and traversal methods.

```rust
use orx_tree::*;

// # A. BUILDING A TREE

//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱ ╲
// 4   5 6   7
// |     |  ╱ ╲
// 8     9 10  11

let mut tree = DynTree::new(1i32);

let mut root = tree.root_mut();
let [id2, id3] = root.push_children([2, 3]);
let [id4, _] = tree.node_mut(&id2).push_children([4, 5]);
let id8 = tree.node_mut(&id4).push_child(8);
let [id6, id7] = tree.node_mut(&id3).push_children([6, 7]);
let id9 = tree.node_mut(&id6).push_child(9);
tree.node_mut(&id7).push_children([10, 11]);
println!("{}", &tree);
// 1
// ├──2
// │  ├──4
// │  │  └──8
// │  └──5
// └──3
//     ├──6
//     │  └──9
//     └──7
//         ├──10
//         └──11

// B. NODE

let node4 = tree.node(&id4);

assert!(!node4.is_leaf());
assert!(!node4.is_root());
assert_eq!(node4.depth(), 2);
assert_eq!(node4.height(), 1);
assert_eq!(node4.sibling_idx(), 0);
assert_eq!(node4.parent(), Some(tree.node(&id2)));
assert_eq!(node4.num_children(), 1);
assert_eq!(node4.get_child(0), Some(tree.node(&id8)));

let ancestors: Vec<_> = node4.ancestors().map(|x| *x.data()).collect();
assert_eq!(ancestors, [2, 1]);

let new_tree: BinaryTree<_> = node4.clone_as_tree();
assert_eq!(new_tree.root().data(), &4);
assert_eq!(new_tree.len(), 2);

// # B. TRAVERSALS

let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

let dfs: Vec<_> = tree.node(&id3).walk::<Dfs>().copied().collect();
assert_eq!(dfs, [3, 6, 9, 7, 10, 11]);

let post_order: Vec<_> = tree.node(&id3).walk::<PostOrder>().copied().collect();
assert_eq!(post_order, [9, 6, 10, 11, 7, 3]);

let leaves: Vec<_> = tree.root().leaves::<Dfs>().copied().collect();
assert_eq!(leaves, [8, 5, 9, 10, 11]);

let node3 = tree.node(&id3);
let paths: Vec<Vec<_>> = node3.paths::<Bfs>().map(|p| p.copied().collect()).collect();
assert_eq!(paths, [[9, 6, 3], [10, 7, 3], [11, 7, 3]]);

let sum: i32 = tree.iter().sum(); // Collection: iterate in arbitrary order
assert_eq!(sum, 66);

for x in tree.iter_mut() { // CollectionMut: iterate in arbitrary order
    *x = 2 * (10 + *x) - *x - 20; // do nothing :)
}

// # C. MUTATIONS - REMOVALS

let mut tree = tree.into_lazy_reclaim(); // to keep the indices valid

// > remove a subtree and collect values in the desired traversal order
let node7 = tree.node_mut(&id7);
let removed_in_bfs_order: Vec<_> = node7.into_walk::<Bfs>().collect();
assert_eq!(removed_in_bfs_order, [7, 10, 11]);
let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(remaining, [1, 2, 3, 4, 5, 6, 8, 9]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱
// 4   5 6
// |     |
// 8     9

// > take out just one node
let node6 = tree.node_mut(&id6);
let taken_out = node6.take_out(); // 6 is removed, 9 moves up
assert_eq!(taken_out, 6);
let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(remaining, [1, 2, 3, 4, 5, 9, 8]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱
// 4   5 9
// |
// 8

// > prune a subtree
let node2 = tree.node_mut(&id2);
let taken_out = node2.prune(); // 2 is removed, together with descendants
assert_eq!(taken_out, 2);
let remaining: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(remaining, [1, 3, 9]);
//      1
//       ╲
//        ╲
//         3
//        ╱
//       9

// # D. MUTATIONS - ADDING & MOVING SUBTREES

// > append another tree as a sibling of a node
let mut other_tree = DynTree::new(2);
let [id4, _] = other_tree.root_mut().push_children([4, 5]);
other_tree.node_mut(&id4).push_child(8);

let id2 = tree
    .node_mut(&id3)
    .push_sibling_tree(Side::Left, other_tree);
let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 2, 3, 4, 5, 9, 8]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱
// 4   5 9
// |
// 8

// > move a subtree to another location in the same tree
let node2 = tree.node(&id2);
let [id4, id5] = [node2.child(0).idx(), node2.child(1).idx()];
tree.node_mut(&id3)
    .push_child_tree_within(id4.into_subtree_within());
let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 2, 3, 5, 9, 4, 8]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//    ╲   ╱ ╲
//     5 9   4
//           |
//           8

// > move the subtree back
tree.node_mut(&id5)
    .push_sibling_tree_within(Side::Left, id4.into_subtree_within());
let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 2, 3, 4, 5, 9, 8]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱
// 4   5 9
// |
// 8

// > insert a node in between parent & child
tree.node_mut(&id9).push_parent(6);
let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 8, 9]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱
// 4   5 6
// |     |
// 8     9

// push a subtree cloned/copied from another tree
let mut other_tree = DynTree::new(100);
let id7 = other_tree.root_mut().push_child(7);
other_tree.node_mut(&id7).push_children([10, 11]);

let subtree = other_tree.node(&id7).as_cloned_subtree();
tree.node_mut(&id3).push_child_tree(subtree);

assert_eq!(other_tree.len(), 4); // unchanged

let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);
//      1
//     ╱ ╲
//    ╱   ╲
//   2     3
//  ╱ ╲   ╱ ╲
// 4   5 6   7
// |     |  ╱ ╲
// 8     9 10  11

// # E. SPLIT TREE INTO TREES

// let's refresh indices
let idx: Vec<_> = tree.root().indices::<Bfs>().collect();
let id2 = idx[1].clone();
let id7 = idx[6].clone();

// let's move subtree rooted at n2 to its own tree
let tree2: DynTree<_> = tree.node_mut(&id2).into_new_tree();
let bfs: Vec<_> = tree2.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [2, 4, 5, 8]);

// let's move subtree rooted at n7 to its own tree, this time a BinaryTree
let tree7: BinaryTree<_> = tree.node_mut(&id7).into_new_tree();
let bfs: Vec<_> = tree7.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [7, 10, 11]);

// these subtrees are moved into new trees; i.e., removed from the original
// alternatively, we could've used 'clone_as_tree' to leave the original tree unchanged
let remaining_bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(remaining_bfs, [1, 3, 6, 9]);
```

### More Examples

* [mutable_recursive_traversal](https://github.com/orxfun/orx-tree/blob/main/examples/mutable_recursive_traversal.rs) demonstrates different approaches to achieve a recursive mutation of all nodes in the tree.

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be added or improved, please open an [issue](https://github.com/orxfun/orx-tree/issues/new) or create a PR.

## License

Dual-licensed under [Apache 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT).
