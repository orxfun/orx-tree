use super::stack::Stack;
use crate::traversal::{
    over::{Over, OverData},
    traverser::Traverser,
};

/// A depth first search traverser ([Wikipedia](https://en.wikipedia.org/wiki/Depth-first_search)).
///
/// A traverser can be created once and used to traverse over trees multiple times without
/// requiring additional memory allocation.
///
/// # Construction
///
/// A depth first traverser can be created,
/// * either by using Default trait and providing its two generic type parameters
///   * `Dfs::<_, OverData>::default()` or `Dfs::<_, OverDepthSiblingIdxData>::default()`, or
///   * `Dfs::<Dyn<u64>, OverData>::default()` or `Dfs::<Dary<2, String>, OverDepthSiblingIdxData>::default()`
///     if we want the complete type signature.
/// * or by using the [`Traversal`] type.
///   * `Traversal.dfs()` or `Traversal.dfs().with_depth().with_sibling_idx()`.
///
/// [`Traversal`]: crate::Traversal
pub struct Dfs<O = OverData>
where
    O: Over,
{
    pub(super) stack: Stack<O::Enumeration>,
}

impl Default for Dfs {
    fn default() -> Self {
        Self::new()
    }
}

impl<O> Traverser<O> for Dfs<O>
where
    O: Over,
{
    type IntoOver<O2>
        = Dfs<O2>
    where
        O2: Over;

    fn new() -> Self {
        Self {
            stack: Default::default(),
        }
    }

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        Dfs::<O2>::new()
    }
}
