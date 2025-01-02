use super::{DfsBfsNodeVal, NodeValueData, NodeValueNode};
use crate::{
    helpers::N,
    iter::{IterMutOver, IterOver, PostNodeVal},
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

/// Iterator over data or values of the nodes.
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
/// // dfs over (depth, sibling index, data)
///
/// let root = tree.root().unwrap();
///
/// for data in root.dfs_over::<OverData>() {
///     // ...
/// }
///
/// let mut iter = root.dfs_over::<OverData>();
/// assert_eq!(iter.next(), Some(&1));
/// assert_eq!(iter.next(), Some(&2));
/// assert_eq!(iter.next(), Some(&4));
/// assert_eq!(iter.next(), Some(&8));
/// assert_eq!(iter.next(), Some(&5)); // ...
///
/// let values: Vec<i32> = root.dfs_over::<OverData>().copied().collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// // dfs over (depth, sibling index, node)
///
/// for node in root.dfs_over::<OverNode>() {
///     // ...
/// }
///
/// let nodes: Vec<BinaryNode<i32>> = root.dfs_over::<OverNode>().collect();
///
/// let values: Vec<i32> = nodes.iter().map(|x| *x.data()).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let num_children: Vec<usize> = nodes.iter().map(|x| x.num_children()).collect();
/// assert_eq!(num_children, [2, 2, 1, 0, 0, 2, 1, 0, 2, 0, 0]);
/// ```
pub struct OverData;

impl IterOver for OverData {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = NodePtr<V>
    where
        V: TreeVariant;
}

impl IterMutOver for OverData {}

/// Iterator directly over tree nodes.
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
/// let mut tree = DynTree::<i32>::new(1);
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
/// // dfs over (depth, sibling index, data)
///
/// let root = tree.root().unwrap();
///
/// for data in root.dfs_over::<OverData>() {
///     // ...
/// }
///
/// let mut iter = root.dfs_over::<OverData>();
/// assert_eq!(iter.next(), Some(&1));
/// assert_eq!(iter.next(), Some(&2));
/// assert_eq!(iter.next(), Some(&4));
/// assert_eq!(iter.next(), Some(&8));
/// assert_eq!(iter.next(), Some(&5)); // ...
///
/// let values: Vec<i32> = root.dfs_over::<OverData>().copied().collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// // dfs over (depth, sibling index, node)
///
/// for node in root.dfs_over::<OverNode>() {
///     // ...
/// }
///
/// let nodes: Vec<DynNode<_>> = root.dfs_over::<OverNode>().collect();
///
/// let values: Vec<i32> = nodes.iter().map(|x| *x.data()).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let num_children: Vec<usize> = nodes.iter().map(|x| x.num_children()).collect();
/// assert_eq!(num_children, [2, 2, 1, 0, 0, 2, 1, 0, 2, 0, 0]);
/// ```
pub struct OverNode;

impl IterOver for OverNode {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = NodePtr<V>
    where
        V: TreeVariant;
}
