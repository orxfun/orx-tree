use super::DfsIterMut;
use crate::iter::{IterMutOver, IterOver};
use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::{helpers::N, iter::DfsIter, node_ref::NodeRefCore, NodeMut, NodeRef, TreeVariant};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_selfref_col::MemoryPolicy;
use orx_split_vec::PinnedVec;

/// An iterable which can create depth-first iterators over and over, using the same only-once allocated stack.
///
/// # Examples
///
/// The following example demonstrates how the iterable created from [`TraversalDepr`] can be used
/// to repeatedly iterate over trees without requiring new allocation.
///
/// [`TraversalDepr`]: crate::TraversalDepr
///
/// ```
/// use orx_tree::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲   ╱ ╲
/// // 4   5 6   7
/// // |     |  ╱ ╲
/// // 8     9 10  11
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
/// let [id2, id3] = root.grow([2, 3]);
///
/// let mut n2 = id2.node_mut(&mut tree);
/// let [id4, _] = n2.grow([4, 5]);
///
/// id4.node_mut(&mut tree).push(8);
///
/// let mut n3 = id3.node_mut(&mut tree);
/// let [id6, id7] = n3.grow([6, 7]);
///
/// id6.node_mut(&mut tree).push(9);
/// id7.node_mut(&mut tree).extend([10, 11]);
///
/// // create the iterable for dfs traversal
/// // that creates the internal stack once
///
/// let mut dfs = TraversalDepr::dfs();
///
/// // repeatedly create iterators from it, without allocation
///
/// let root = tree.root().unwrap();
/// let values: Vec<_> = dfs.iter(&root).copied().collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let mut n7 = id7.node_mut(&mut tree);
/// for (i, value) in dfs.iter_mut(&mut n7).enumerate() {
///     *value += (i * 100) as i32;
/// }
///
/// let n3 = id3.node(&tree);
/// let values: Vec<_> = dfs.iter(&n3).copied().collect();
/// assert_eq!(values, [3, 6, 9, 7, 110, 211]);
///
/// let n7 = id7.node(&tree);
/// let values: Vec<_> = dfs.iter(&n7).copied().collect();
/// assert_eq!(values, [7, 110, 211]);
/// ```
pub struct DfsIterable<
    K: IterOver,
    V: TreeVariant,
    M: MemoryPolicy<V> = DefaultMemory<V>,
    P: PinnedVec<N<V>> = DefaultPinVec<V>,
> {
    stack: Vec<K::DfsBfsQueueElement<V>>,
    phantom: PhantomData<(M, P)>,
}

impl<K, V, M, P> Default for DfsIterable<K, V, M, P>
where
    V: TreeVariant,
    K: IterOver,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn default() -> Self {
        Self {
            stack: Vec::new(),
            phantom: PhantomData,
        }
    }
}

impl<K, V, M, P> DfsIterable<K, V, M, P>
where
    V: TreeVariant,
    K: IterOver,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Creates a depth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter<'a>(&'a mut self, root: &'a impl NodeRef<'a, V, M, P>) -> DfsIterOf<'a, V, K, M, P>
    where
        V: 'a,
    {
        DfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.stack)
    }

    /// Creates a mutable depth-first iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_mut<'a>(
        &'a mut self,
        root: &'a mut NodeMut<'a, V, M, P>,
    ) -> DfsIterMutOf<'a, V, K, M, P>
    where
        V: 'a,
        K: IterMutOver,
    {
        DfsIter::new_using(root.col(), root.node_ptr().clone(), &mut self.stack).into()
    }
}

// type simplification of iterators

type DfsIterOf<'a, V, K, M, P> = DfsIter<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut Vec<<K as IterOver>::DfsBfsQueueElement<V>>,
>;

type DfsIterMutOf<'a, V, K, M, P> = DfsIterMut<
    'a,
    <K as IterOver>::DfsBfsIterKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut Vec<<K as IterOver>::DfsBfsQueueElement<V>>,
>;
