use crate::traversal::enumeration::Enumeration;
use alloc::collections::VecDeque;
use orx_selfref_col::NodePtr;

pub type Item<V, E> = <E as Enumeration>::Item<NodePtr<V>>;

pub type Queue<V, E> = VecDeque<Item<V, E>>;
