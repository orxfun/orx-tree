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

#[cfg(any(test, feature = "std"))]
extern crate std;

extern crate alloc;

/// Module defining tree traversal iterators.
pub mod iter;

mod as_tree_node;
mod common_traits;
mod dary;
mod r#dyn;
mod helpers;
mod node;
mod node_mut;
mod node_ref;
mod tree;
mod tree_variant;

pub use as_tree_node::AsTreeNode;
pub use dary::{BinaryNode, BinaryTree, Dary, DaryNode, DaryTree};
pub use iter::Traversal;
pub use node::Node;
pub use node_mut::{NodeMut, NodeMutDown, NodeMutOrientation, NodeMutUpAndDown};
pub use node_ref::NodeRef;
pub use r#dyn::{Dyn, DynNode, DynTree};
pub use tree::Tree;
pub use tree_variant::TreeVariant;
