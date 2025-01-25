# orx-tree

[![orx-tree crate](https://img.shields.io/crates/v/orx-tree.svg)](https://crates.io/crates/orx-tree)
[![orx-tree documentation](https://docs.rs/orx-tree/badge.svg)](https://docs.rs/orx-tree)

A beautiful tree 🌳 with convenient and efficient growth, mutation and traversal features.

## Features

### Generic Variants

[`Tree`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html) is generic over variants that define the way the children are stored:

* [`DynTree<T>`](https://docs.rs/orx-tree/latest/orx_tree/type.DynTree.html), or equivalently `Tree<Dyn<T>>`, is a tree where each node can contain any number of children stored as a vector.
* [`DaryTree<D, T>`](https://docs.rs/orx-tree/latest/orx_tree/type.DaryTree.html), or equivalently `Tree<DaryTree<D, T>>`, is a tree where each node can contain at most `D` children stored inlined as an array.
  * [`BinaryTree<T>`](https://docs.rs/orx-tree/latest/orx_tree/type.BinaryTree.html) is simply a shorthand for `DaryTree<2, T>`.

### Recursive Nature of Trees

Note that [`Tree`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html) has only few methods which mainly allow access to the root or to any node using node indices. Since every node represents a subtree rooted at itself, the core tree functionalities are provided as methods of [`NodeRef`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html) and [`NodeMut`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html), which are immutable and mutable nodes, respectively.

### Traversals

We can walk all nodes of a subtree rooted at any node using a generic traversal parameter. For instance, let `node` be a node of the tree, then:

* [`node.walk::<Bfs>()`](https://docs.rs/orx-tree/latest/orx_tree/traversal/struct.Bfs.html) creates an iterator that visits all the nodes belonging to the subtree rooted at the *node* in the breadth-first order.
* [`node.walk_mut::<Dfs>()`](https://docs.rs/orx-tree/latest/orx_tree/traversal/struct.Dfs.html) creates a mutable iterator, this time in the (pre-order) depth-first order.
* [`node_into_walk::<PostOrder>()`](https://docs.rs/orx-tree/latest/orx_tree/traversal/struct.PostOrder.html), on the other hand, takes the subtree rooted at the *node* out of the tree and yields the elements in post-order.

### Special Iterators

Abovementioned traverser kinds can be used to create other specialized iterators as well:

* [`node.leaves::<Bfs>()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.leaves) yields the leaf nodes in the subtree rooted at *node* in breadth-first order.
* [`node.paths::<Dfs>()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.paths) yields all the paths or sequences of nodes connecting the *node* to all of its leaves in the depth-first order.

On the other hand, [`node.ancestors()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.ancestors) provides an upward iterator from the *node* to the root of the tree.

We also can walk the tree in an alternative desired order by using methods such as:

* [`node.child(child_idx)`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.child), [`node.children()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.children), [`node.children_mut()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.children_mut), [`node.into_child(child_idx)`]((https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.into_child))
* [`node.parent()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.parent), [`node.into_parent()`](https://docs.rs/orx-tree/latest/orx_tree/trait.NodeRef.html#method.into_parent), etc.

The tree naturally implements [`Collection`](https://docs.rs/orx-iterable/latest/orx_iterable/trait.Collection.html) and [`CollectionMut`](https://docs.rs/orx-iterable/latest/orx_iterable/trait.CollectionMut.html) providing iterators via `iter` and `iter_mut` methods. Since the tree is not a linear data structure, these iterators yield elements in an arbitrary (but deterministic) order. The following are some example cases where the traversal order is not important, and hence, these iterators are useful:

* `iter_mut` to map data of node; for instance, to double values of all nodes which happen to have an odd value.
* `iter` to make reductions; for instance, to get the sum of values of all nodes in a subtree.

### Constant Time Access to Nodes via Node Indices

A [`NodeIdx`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeIdx.html) for a [`Tree`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html) is similar to `usize` for a slice in that it allows constant time access to the node it is created for.

On the other hand, it is more specific for the node due to the following:

* usize represents a position of the slice. Say we have the slice *[a, b, c]*. Currently, index 0 points to element *a*. However, if we swap the first and third elements, index 0 will now be pointing to *c* because the usize represents a position on the slice.
* A node index represents the node it is created for. If the index is created for node *a*, it will always point to this node no matter how many times we move the node in the tree. Further, we cannot use this node index on another tree and it does not allow access to another node if node *a* is removed from the tree.

### Cache Locality

Nodes of the tree are stored in an underlying [`PinnedVec`](https://crates.io/crates/orx-pinned-vec) with pinned element guarantees. This allows for keeping the nodes close to each other improving cache locality while still providing with constant time mutation methods.

### Convenient Mutations

#### Growth & Move Subtrees Around

There exist five methods that adds descendants to a node:

* [`push_child(value)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child) => adds a single child
* [`push_children(values)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_children) => adds a constant number of children
* [`extend_children(values)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.extend_children) => adds a variable number of children provided by an iterator
* [`push_child_tree(subtree)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child_tree) => appends the subtree as descendants of the *node* such that the root of the subtree is the child of the *node*
* [`push_child_tree_within(subtree)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_child_tree_within) => similar to the above except that the subtree belongs to the same tree, we might be moving or cloning the subtree

These methods have the *sibling* variants such as [`push_sibling`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_sibling) rather than *push_child* which allows to define the side of the new sibling.

Further, [`push_parent(value)`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.push_parent) allows to push a node in between a node and its parent.

All together, these methods allow to insert nodes or subtrees at any position of the tree.

Note that all the growth methods return the indices of the created nodes allowing for a fluent growth of the tree.

Finally, the tree provides methods to [`swap_subtrees`](https://docs.rs/orx-tree/latest/orx_tree/struct.Tree.html#method.swap_subtrees) withing the tree.

#### Removals

We can take out a node from the tree, while connecting its parent to its children via the [`take_out`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.take_out) method.

Alternatively, we can [`prune`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.prune) by removing a subtree rooted at a particular node, and returns the value of the root node.

If we need the data of all nodes of the removed subtree, we can create an [`into_walk`](https://docs.rs/orx-tree/latest/orx_tree/struct.NodeMut.html#method.into_walk) iterator from the node which will both remove the subtree and yield the data of removed nodes in the selected traversal order.

## Opt-in Features

* **std**: This is a no-std crate by default, and hence, "std" feature needs to be included when necessary.
* **serde**: Tree implements `Serialize` and `Deserialize` traits; the "serde" feature needs to be added when required. It uses a linearized representation of the tree as a [`DepthFirstSequence`](https://docs.rs/orx-tree/latest/orx_tree/struct.DepthFirstSequence.html). You may find de-serialization examples in the corresponding [test file](https://github.com/orxfun/orx-tree/blob/main/tests/serde.rs).

# Example

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
assert_eq!(ancestors, [4, 2, 1]);

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
let tree7: DynTree<_> = tree.node_mut(&id7).into_new_tree();
let bfs: Vec<_> = tree7.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [7, 10, 11]);

// these subtrees are moved into new trees; i.e., removed from the original
// alternatively, we could've used 'clone_as_tree' to leave the original tree unchanged
let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
assert_eq!(bfs, [1, 3, 6, 9]);
```

## Contributing

Contributions are welcome! If you notice an error, have a question or think something could be added or improved, please open an [issue](https://github.com/orxfun/orx-tree/issues/new) or create a PR.

## License

This library is licensed under MIT license. See LICENSE for details.
