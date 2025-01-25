use super::{breadth_first::Bfs, depth_first::Dfs, post_order::PostOrder};

/// Type with methods allowing to create different [`Traverser`] types with
/// different walk strategies, such as depth-first, breadth-first or post-order.
///
/// [`Traverser`]: crate::traversal::Traverser
#[derive(Clone, Copy)]
pub struct Traversal;

impl Traversal {
    /// Creates the default (pre-order) depth-first-search traverser
    /// ([Wikipedia](https://en.wikipedia.org/wiki/Depth-first_search)).
    ///
    /// The default traverser creates iterators that yield references or mutable references
    /// to the node data; i.e., [`OverData`].
    ///
    /// However, item type of the iterators that the traverser creates can be transformed
    /// any time using the transformation methods:
    ///
    /// * [`over_data`]
    /// * [`over_nodes`]
    /// * [`with_depth`]
    /// * [`with_sibling_idx`]
    ///
    /// [`OverData`]: crate::traversal::OverData
    /// [`over_data`]: crate::traversal::Traverser::over_data
    /// [`over_nodes`]: crate::traversal::Traverser::over_nodes
    /// [`with_depth`]: crate::traversal::Traverser::with_depth
    /// [`with_sibling_idx`]: crate::traversal::Traverser::with_sibling_idx
    pub fn dfs(self) -> Dfs {
        Default::default()
    }

    /// Creates the default breadth-first-search traverser, also known as level-order
    /// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search))
    ///
    /// The default traverser creates iterators that yield references or mutable references
    /// to the node data; i.e., [`OverData`].
    ///
    /// However, item type of the iterators that the traverser creates can be transformed
    /// any time using the transformation methods:
    ///
    /// * [`over_data`]
    /// * [`over_nodes`]
    /// * [`with_depth`]
    /// * [`with_sibling_idx`]
    ///
    /// [`OverData`]: crate::traversal::OverData
    /// [`over_data`]: crate::traversal::Traverser::over_data
    /// [`over_nodes`]: crate::traversal::Traverser::over_nodes
    /// [`with_depth`]: crate::traversal::Traverser::with_depth
    /// [`with_sibling_idx`]: crate::traversal::Traverser::with_sibling_idx
    pub fn bfs(self) -> Bfs {
        Default::default()
    }

    /// Creates the default post-order traverser
    /// ([Wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
    ///
    /// The default traverser creates iterators that yield references or mutable references
    /// to the node data; i.e., [`OverData`].
    ///
    /// However, item type of the iterators that the traverser creates can be transformed
    /// any time using the transformation methods:
    ///
    /// * [`over_data`]
    /// * [`over_nodes`]
    /// * [`with_depth`]
    /// * [`with_sibling_idx`]
    ///
    /// [`OverData`]: crate::traversal::OverData
    /// [`over_data`]: crate::traversal::Traverser::over_data
    /// [`over_nodes`]: crate::traversal::Traverser::over_nodes
    /// [`with_depth`]: crate::traversal::Traverser::with_depth
    /// [`with_sibling_idx`]: crate::traversal::Traverser::with_sibling_idx
    pub fn post_order(self) -> PostOrder {
        Default::default()
    }
}
