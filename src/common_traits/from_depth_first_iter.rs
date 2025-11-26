use crate::{MemoryPolicy, Tree, TreeVariant, pinned_storage::PinnedStorage};

/// A depth first sequence is a representation of a tree in a linear storage of (depth, value) tuples.
/// This is useful in collecting trees from iterators, (de)-serializing trees or converting its variant
/// from one to another.
///
/// `DepthFirstSequence` struct is nothing but a wrapper around a `(usize, T)` iterator in order
/// to state explicitly that this iterator is expected to follow the depth-first-order of (depth, value) pairs.
///
/// A `DepthFirstSequence` can be created from any type that implements `IntoIterator<Item = (usize, T)>`
/// using `From` (or `Into`) traits.
///
/// In order to create a valid tree from the iterator, the order of pairs must satisfy certain conditions.
/// Assume (depth(i), value(i)) is the i-th item of the iterator.
/// Then, the following conditions summarize the valid relation between successive elements of the iterator:
///
/// * depth(0) = 0
///   * since the first node of the depth-first traversal is the root
/// * depth(i+1) < depth(i) is valid
///   * we are moving from a leaf node with depth(i) to next child of one of its ancestors
/// * depth(i+1) = depth(i) is valid
///   * we are moving from a leaf node to its sibling which is immediately right to it
/// * depth(i+1) = depth(i) + 1 is valid
///   * we are moving from a non-leaf node to its first child
///
/// On the contrary, if either of the following two conditions hold, we cannot build a valid tree.
///
/// * depth(0) > 0
///   * leads to [`DepthFirstSequenceError::NonZeroRootDepth`]
/// * depth(i + 1) = depth(i) + q where q > 1
///   * leads to [`DepthFirstSequenceError::DepthIncreaseGreaterThanOne`]
///
/// If either of these conditions hold, `try_from` or `try_into` methods will return the corresponding
/// error instead of a valid tree.
///
/// # Examples
///
/// ## Happy Paths
///
/// The following examples demonstrate the happy paths leading to successful collection of a tree from valid
/// depth-first sequences.
///
/// ```
/// use orx_tree::*;
///
/// // empty tree
///
/// let dfs = DepthFirstSequence::from([]);
/// let result: Result<DynTree<u32>, DepthFirstSequenceError> = dfs.try_into();
/// assert_eq!(result, Ok(Tree::empty()));
///
/// // non-empty tree
///
/// //      0
/// //     ╱ ╲
/// //    ╱   ╲
/// //   1     2
/// //  ╱     ╱ ╲
/// // 3     4   5
/// // |         |
/// // 6         7
/// let depth_value_pairs = [
///     (0, 0),
///     (1, 1),
///     (2, 3),
///     (3, 6),
///     (1, 2),
///     (2, 4),
///     (2, 5),
///     (3, 7),
/// ];
/// let dfs = DepthFirstSequence::from(depth_value_pairs.clone());
/// let result: Result<DynTree<u32>, DepthFirstSequenceError> = dfs.try_into();
///
/// assert!(result.is_ok());
/// let tree = result.unwrap();
///
/// let bfs: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
/// assert_eq!(bfs, [0, 1, 2, 3, 4, 5, 6, 7]);
///
/// // we can get back the dfs-sequence from constructed tree using walk_with
///
/// let mut t = Traversal.dfs().with_depth();
/// let dfs_back_from_tree: Vec<_> = tree
///     .root()
///     .walk_with(&mut t)
///     .map(|(depth, val)| (depth, *val))
///     .collect();
/// assert_eq!(dfs_back_from_tree, depth_value_pairs);
///
/// // we can construct back any fitting tree variant from the sequence
///
/// let result = DepthFirstSequence::from(dfs_back_from_tree).try_into();
/// assert!(result.is_ok());
///
/// let tree_back: BinaryTree<u32> = result.unwrap();
/// assert_eq!(tree, tree_back);
/// ```
///
/// ## Error Paths
///
/// The following examples illustrate the two potential error cases that can be observed due to
/// the iterator not yielding a valid depth-first sequence.
///
/// ```
/// use orx_tree::*;
///
/// // root with depth > 0
///
/// let dfs = DepthFirstSequence::from([(1, 1)]);
/// let result: Result<DynTree<u32>, DepthFirstSequenceError> = dfs.try_into();
/// assert_eq!(result, Err(DepthFirstSequenceError::NonZeroRootDepth));
///
/// // missing node (or forgotten depth) in the sequence
///
/// //       0
/// //      ╱ ╲
/// //     ╱   ╲
/// //    1     2
/// //   ╱     ╱ ╲
/// // ???    4   5
/// //  |         |
/// //  6         7
/// let depth_value_pairs = [
///     (0, 0),
///     (1, 1),
///     // (2, 3), -> forgotten node leads to depth jump from 1 to 3
///     (3, 6),
///     (1, 2),
///     (2, 4),
///     (2, 5),
///     (3, 7),
/// ];
/// let dfs = DepthFirstSequence::from(depth_value_pairs.clone());
/// let result: Result<DynTree<u32>, DepthFirstSequenceError> = dfs.try_into();
/// assert_eq!(
///     result,
///     Err(DepthFirstSequenceError::DepthIncreaseGreaterThanOne {
///         depth: 1,
///         succeeding_depth: 3
///     })
/// );
/// ```
#[derive(Clone)]
pub struct DepthFirstSequence<T, I>(I)
where
    I: IntoIterator<Item = (usize, T)>;

impl<T, I> From<I> for DepthFirstSequence<T, I>
where
    I: IntoIterator<Item = (usize, T)>,
{
    fn from(iter: I) -> Self {
        Self(iter)
    }
}

/// A depth first sequence, or [`DepthFirstSequence`] is simply a sequence of `(usize, T)` tuples
/// corresponding to (depth, value) pairs of nodes of a tree which are ordered by the depth-first
/// traversal order.
///
/// Therefore, not all `IntoIterator<Item = (usize, T)>` types satisfy the depth-first sequence
/// requirement.
/// The invalid sequences are represented by the `DepthFirstSequenceError` type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DepthFirstSequenceError {
    /// The first element of the depth-first sequence must be the root with depth 0.
    /// Therefore, any sequence with a first element having a non-zero depth leads to this error.
    ///
    /// Note that empty sequences are valid and represent an empty tree.
    NonZeroRootDepth,
    /// While traversing a tree in depth first order, we
    ///
    /// * either move one level down to access the child (depth = previous depth + 1)
    /// * or stay at the same level to access the sibling to the right (depth = previous depth)
    /// * or move up and then right to access the next child of an ancestor (depth < previous depth)
    ///
    /// This list represents valid depth transition.
    /// However, we never
    ///
    /// * move n > 1 level down (depth > previous depth + 1)
    ///
    /// This leaves a gap in the depth-first traversal, and hance, is the invalid case that this
    /// error variant represents.
    DepthIncreaseGreaterThanOne {
        /// Depth of the node where the error is observed.
        depth: usize,
        /// Depth succeeding the `depth` which is at least two more than the previous.
        succeeding_depth: usize,
    },
}

impl<I, V, M, P> TryFrom<DepthFirstSequence<V::Item, I>> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
    I: IntoIterator<Item = (usize, V::Item)>,
{
    type Error = DepthFirstSequenceError;

    /// Tries to convert a depth-first sequence into a valid tree.
    /// Returns the corresponding [`DepthFirstSequenceError`] if the sequence is invalid.
    ///
    /// Note that:
    ///
    /// * A [`DepthFirstSequence`] is just a wrapper of any `IntoIterator<Item = (usize, T)>` implementor
    ///   that can be crated using the `From` trait => `DepthFirstSequence::from([(0, "a"), (1, "b")])`.
    /// * However, not all `IntoIterator<Item = (usize, T)>` instances represent a valid depth first
    ///   sequence. Therefore, the conversion is fallible.
    fn try_from(value: DepthFirstSequence<V::Item, I>) -> Result<Self, Self::Error> {
        let mut iter = value.0.into_iter();
        match iter.next() {
            None => Ok(Tree::default()),
            Some((d, root)) => match d {
                0 => {
                    let mut tree = Tree::new_with_root(root);
                    let mut peekable = iter.peekable();
                    match peekable.peek().is_some() {
                        true => match tree.root_mut().try_append_subtree_as_child(peekable, 0) {
                            Ok(_) => Ok(tree),
                            Err((depth, succeeding_depth)) => {
                                Err(DepthFirstSequenceError::DepthIncreaseGreaterThanOne {
                                    depth,
                                    succeeding_depth,
                                })
                            }
                        },
                        false => Ok(tree),
                    }
                }
                _ => Err(DepthFirstSequenceError::NonZeroRootDepth),
            },
        }
    }
}
