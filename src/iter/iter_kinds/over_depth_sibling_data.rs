use super::{node_value::NodeValuePtr, DfsBfsNodeDepthSiblingVal, NodeValueData, NodeValueNode};
use crate::{
    helpers::N,
    iter::{IterMutOver, IterOver, PostNodeDepthSiblingVal},
    TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

/// Iterator over (node depth, index of node among its siblings, node data) tuples.
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
/// for (depth, sibling_idx, data) in root.dfs_over::<OverDepthSiblingData>() {
///     // ...
/// }
///
/// let mut iter = root.dfs_over::<OverDepthSiblingData>();
/// assert_eq!(iter.next(), Some((0, 0, &1)));
/// assert_eq!(iter.next(), Some((1, 0, &2)));
/// assert_eq!(iter.next(), Some((2, 0, &4)));
/// assert_eq!(iter.next(), Some((3, 0, &8)));
/// assert_eq!(iter.next(), Some((2, 1, &5))); // ...
///
/// let all: Vec<(usize, usize, &i32)> = root.dfs_over::<OverDepthSiblingData>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
/// assert_eq!(sibling_indices, [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.2).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// // dfs over (depth, sibling index, node)
///
/// let all: Vec<(usize, usize, Node<_>)> = root.dfs_over::<OverDepthSiblingNode>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
/// assert_eq!(sibling_indices, [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.2.data()).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let num_children: Vec<usize> = all.iter().map(|x| x.2.num_children()).collect();
/// assert_eq!(num_children, [2, 2, 1, 0, 0, 2, 1, 0, 2, 0, 0]);
/// ```
pub struct OverDepthSiblingData;

impl IterOver for OverDepthSiblingData {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeDepthSiblingVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeDepthSiblingVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = (usize, usize, NodePtr<V>)
    where
        V: TreeVariant;
}

impl IterMutOver for OverDepthSiblingData {}

/// Iterator over (node depth, index of node among its siblings, node) tuples.
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
/// for (depth, sibling_idx, data) in root.dfs_over::<OverDepthSiblingData>() {
///     // ...
/// }
///
/// let mut iter = root.dfs_over::<OverDepthSiblingData>();
/// assert_eq!(iter.next(), Some((0, 0, &1)));
/// assert_eq!(iter.next(), Some((1, 0, &2)));
/// assert_eq!(iter.next(), Some((2, 0, &4)));
/// assert_eq!(iter.next(), Some((3, 0, &8)));
/// assert_eq!(iter.next(), Some((2, 1, &5))); // ...
///
/// let all: Vec<(usize, usize, &i32)> = root.dfs_over::<OverDepthSiblingData>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
/// assert_eq!(sibling_indices, [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.2).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// // dfs over (depth, sibling index, node)
///
/// let all: Vec<(usize, usize, Node<_>)> = root.dfs_over::<OverDepthSiblingNode>().collect();
///
/// let depths: Vec<usize> = all.iter().map(|x| x.0).collect();
/// assert_eq!(depths, [0, 1, 2, 3, 2, 1, 2, 3, 2, 3, 3]);
///
/// let sibling_indices: Vec<usize> = all.iter().map(|x| x.1).collect();
/// assert_eq!(sibling_indices, [0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1]);
///
/// let values: Vec<i32> = all.iter().map(|x| *x.2.data()).collect();
/// assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);
///
/// let num_children: Vec<usize> = all.iter().map(|x| x.2.num_children()).collect();
/// assert_eq!(num_children, [2, 2, 1, 0, 0, 2, 1, 0, 2, 0, 0]);
/// ```
pub struct OverDepthSiblingNode;

impl IterOver for OverDepthSiblingNode {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeDepthSiblingVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeDepthSiblingVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = (usize, usize, NodePtr<V>)
    where
        V: TreeVariant;
}

/// Iterator over (node depth, index of node among its siblings, node pointer) tuples.
pub struct OverDepthSiblingPtr;

impl IterOver for OverDepthSiblingPtr {
    type DfsBfsIterKind<'a, V, M, P>
        = DfsBfsNodeDepthSiblingVal<NodeValuePtr>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type PostOrderKind<'a, V, M, P>
        = PostNodeDepthSiblingVal<NodeValuePtr>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type DfsBfsQueueElement<V>
        = (usize, usize, NodePtr<V>)
    where
        V: TreeVariant;
}
