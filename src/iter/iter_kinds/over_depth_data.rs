use super::{node_value::NodeValuePtr, DfsBfsNodeDepthVal, NodeValueData, NodeValueNode};
use crate::{
    helpers::N,
    iter::{IterMutOver, IterOver, PostNodeDepthVal},
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

/// Iterator over (node depth, node data) pairs.
///
/// # Examples
///
/// ```
/// use orx_tree::*;
/// use orx_tree::iter::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲   ╱ ╲
/// // 4   5 6   7
/// // |     |  ╱ ╲
/// // 8     9 10  11
/// let mut tree = BinaryTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
/// root.extend([2, 3]);
///
/// let mut n2 = root.into_child_mut(0).unwrap();
/// n2.extend([4, 5]);
///
/// let mut n4 = n2.into_child_mut(0).unwrap();
/// n4.push(8);
///
/// let mut n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
/// n3.extend([6, 7]);
///
/// let mut n6 = n3.into_child_mut(0).unwrap();
/// n6.push(9);
///
/// let mut n7 = n6.into_parent_mut().unwrap().into_child_mut(1).unwrap();
/// n7.extend([10, 11]);
///
/// // dfs over (depth, data)
///
/// let root = tree.root().unwrap();
///
/// for (depth, data) in root.dfs_over::<OverDepthData>() {
///     // ...
/// }
///
/// let mut iter = root.dfs_over::<OverDepthData>();
/// assert_eq!(iter.next(), Some((0, &1)));
/// assert_eq!(iter.next(), Some((1, &2)));
/// assert_eq!(iter.next(), Some((2, &4)));
/// assert_eq!(iter.next(), Some((3, &8)));
/// assert_eq!(iter.next(), Some((2, &5))); // ...
///
/// let all: Vec<(usize, &i32)> = root.dfs_over::<OverDepthData>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.1).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// // dfs over (depth, node)
///
/// let all: Vec<(usize, Node<_>)> = root.dfs_over::<OverDepthNode>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.1.data()).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let num_children: Vec<usize> = all.iter().map(|x| x.1.num_children()).collect();
/// assert_eq!(num_children, [2, 2, 1, 0, 0, 2, 1, 0, 2, 0, 0]);
/// ```
pub struct OverDepthData;

impl IterOver for OverDepthData {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeDepthVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeDepthVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = (usize, NodePtr<V>)
    where
        V: TreeVariant;
}

impl IterMutOver for OverDepthData {}

/// Iterator over (node depth, node) pairs.
///
/// # Examples
///
/// ```
/// use orx_tree::*;
/// use orx_tree::iter::*;
///
/// //      1
/// //     ╱ ╲
/// //    ╱   ╲
/// //   2     3
/// //  ╱ ╲   ╱ ╲
/// // 4   5 6   7
/// // |     |  ╱ ╲
/// // 8     9 10  11
/// let mut tree = BinaryTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
/// root.extend([2, 3]);
///
/// let mut n2 = root.into_child_mut(0).unwrap();
/// n2.extend([4, 5]);
///
/// let mut n4 = n2.into_child_mut(0).unwrap();
/// n4.push(8);
///
/// let mut n3 = tree.root_mut().unwrap().into_child_mut(1).unwrap();
/// n3.extend([6, 7]);
///
/// let mut n6 = n3.into_child_mut(0).unwrap();
/// n6.push(9);
///
/// let mut n7 = n6.into_parent_mut().unwrap().into_child_mut(1).unwrap();
/// n7.extend([10, 11]);
///
/// // dfs over (depth, data)
///
/// let root = tree.root().unwrap();
///
/// for (depth, data) in root.dfs_over::<OverDepthData>() {
///     // ...
/// }
///
/// let mut iter = root.dfs_over::<OverDepthData>();
/// assert_eq!(iter.next(), Some((0, &1)));
/// assert_eq!(iter.next(), Some((1, &2)));
/// assert_eq!(iter.next(), Some((2, &4)));
/// assert_eq!(iter.next(), Some((3, &8)));
/// assert_eq!(iter.next(), Some((2, &5))); // ...
///
/// let all: Vec<(usize, &i32)> = root.dfs_over::<OverDepthData>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.1).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// // dfs over (depth, node)
///
/// let all: Vec<(usize, Node<_>)> = root.dfs_over::<OverDepthNode>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.1.data()).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let num_children: Vec<usize> = all.iter().map(|x| x.1.num_children()).collect();
/// assert_eq!(num_children, [2, 2, 1, 0, 0, 2, 1, 0, 2, 0, 0]);
/// ```
pub struct OverDepthNode;

impl IterOver for OverDepthNode {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeDepthVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeDepthVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = (usize, NodePtr<V>)
    where
        V: TreeVariant;
}

/// Iterator over (node depth, node pointer) pairs.
pub struct OverDepthPtr;

impl IterOver for OverDepthPtr {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeDepthVal<NodeValuePtr>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeDepthVal<NodeValuePtr>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = (usize, NodePtr<V>)
    where
        V: TreeVariant;
}
