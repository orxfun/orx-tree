#[cfg(test)]
mod tests;

mod dfs_enumeration;
pub(crate) mod iter_mut;
pub(crate) mod iter_ptr;
pub(crate) mod iter_ref;
mod traverser;

pub type Item<V, E> =
    <E as crate::traversal::enumeration::Enumeration>::Item<orx_selfref_col::NodePtr<V>>;
pub type Stack<V, E> = alloc::vec::Vec<Item<V, E>>;

pub use dfs_enumeration::DepthFirstEnumeration;
pub use traverser::Dfs;
