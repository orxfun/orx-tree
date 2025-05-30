#![doc = include_str!("../README.md")]
#![warn(
    missing_docs,
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

#[cfg(test)]
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

mod aliases;
mod common_traits;
mod dary;
mod r#dyn;
mod errors;
mod node;
mod node_mut;
mod node_ref;
mod subtrees;
mod subtrees_within;
mod tree;
mod tree_node_idx;
mod tree_variant;

pub use common_traits::{DepthFirstSequence, DepthFirstSequenceError};
pub use dary::{Binary, BinaryNode, BinaryTree, Dary, DaryNode, DaryTree};
pub use r#dyn::{Dyn, DynNode, DynTree};
pub use memory::{Auto, AutoWithThreshold, Lazy, MemoryPolicy};
pub use node::Node;
pub use node_mut::{NodeMut, NodeMutDown, NodeMutOrientation, NodeMutUpAndDown, Side};
pub use node_ref::NodeRef;
pub use subtrees::SubTree;
pub use traversal::{Bfs, Dfs, PostOrder, Traversal, Traverser};
pub use tree::Tree;
pub use tree_node_idx::NodeIdx;
pub use tree_variant::TreeVariant;

// ERRORS
pub use errors::NodeSwapError;
pub use orx_selfref_col::NodeIdxError;

// RE-IMPORT
pub use orx_iterable::{Collection, CollectionMut};

#[cfg(feature = "orx-parallel")]
pub use orx_parallel::*;
