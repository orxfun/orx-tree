#[cfg(test)]
mod tests;

mod dfs_core;
mod dfs_element;
mod iter_mut;
mod iter_ptr;
mod iter_ref;

pub use iter_ptr::DfsIterPtr;

pub type Item<V, E> = <E as crate::traversal::Element>::Item<orx_selfref_col::NodePtr<V>>;
pub type Stack<V, E> = alloc::vec::Vec<Item<V, E>>;
