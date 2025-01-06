use crate::traversal::enumeration::Enumeration;
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

pub type Item<V, E> = <E as Enumeration>::Item<NodePtr<V>>;

pub type Stack<V, E> = Vec<Item<V, E>>;
