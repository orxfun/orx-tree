#[cfg(test)]
mod tests;

mod dfs_enumeration;
mod iter_mut;
mod iter_ptr;
mod iter_ref;
mod traverser;

pub type Item<V, E> = <E as crate::traversal::Enumeration>::Item<orx_selfref_col::NodePtr<V>>;
pub type Stack<V, E> = alloc::vec::Vec<Item<V, E>>;

pub use traverser::Dfs;
