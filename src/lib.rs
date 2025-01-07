#![doc = include_str!("../README.md")]
#![warn(
    // missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::float_cmp,
    clippy::float_cmp_const,
    clippy::missing_panics_doc,
    clippy::todo
)]
#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

extern crate alloc;

/// Module defining iterators.
pub mod iter;

/// Module defining tree traversal iterators.
pub mod traversal;

/// Module defining memory reclaim policies.
pub mod memory;

/// Module defining the choice over the pinned storage of the tree.
pub mod pinned_storage;

mod as_tree_node;
mod common_traits;
mod dary;
mod r#dyn;
mod helpers;
mod node;
mod node_mut;
mod node_ref;
mod tree;
mod tree_node_idx;
mod tree_variant;

pub use as_tree_node::AsTreeNode;
pub use dary::{BinaryNode, BinaryTree, Dary, DaryNode, DaryTree};
pub use memory::{Auto, AutoWithThreshold, Lazy, TreeMemoryPolicy};
pub use node::Node;
pub use node_mut::{NodeMut, NodeMutDown, NodeMutOrientation, NodeMutUpAndDown};
pub use node_ref::NodeRef;
pub use r#dyn::{Dyn, DynNode, DynTree};
pub use traversal::{Traversal, Traverser, TraverserMut};
pub use tree::Tree;
pub use tree_node_idx::NodeIdx;
pub use tree_variant::TreeVariant;

// re-import from self-ref-col
pub use orx_selfref_col::NodeIdxError;
