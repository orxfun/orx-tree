use super::{depth_first::Dfs, post_order::PostOrder};
use crate::TreeVariant;

#[derive(Clone, Copy)]
pub struct Traversal;

impl Traversal {
    pub fn dfs<V: TreeVariant>(self) -> Dfs<V> {
        Default::default()
    }

    pub fn post_order<V: TreeVariant>(self) -> PostOrder<V> {
        Default::default()
    }
}
