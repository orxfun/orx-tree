use super::{
    kind_traits::{node, node_mut, IterMutOver},
    IterKindCore, IterOver, NodeValue, NodeValueData, NodeValueNode, QueueElement,
};
use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

// core

/// Iterator over tuples of node depths and values obtained from tree nodes.
pub struct NodeDepthVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterKindCore<'a, V, M, P> for NodeDepthVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: NodeValue<'a, V, M, P>,
{
    type QueueElement = (usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (
        usize,
        <Self::ValueFromNode as NodeValue<'a, V, M, P>>::Value,
    );

    type YieldElementMut = (
        usize,
        <Self::ValueFromNode as NodeValue<'a, V, M, P>>::ValueMut,
    );

    #[inline(always)]
    fn children(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .map(move |ptr| (depth, ptr.clone()))
    }

    #[inline(always)]
    fn children_rev(parent: &Self::QueueElement) -> impl Iterator<Item = Self::QueueElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .rev()
            .map(move |ptr| (depth, ptr.clone()))
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElement {
        (queue_element.0, D::value(col, node(&queue_element.1)))
    }

    fn element_mut(
        col: &'a SelfRefCol<V, M, P>,
        queue_element: &Self::QueueElement,
    ) -> Self::YieldElementMut {
        (
            queue_element.0,
            D::value_mut(col, node_mut(&queue_element.1)),
        )
    }
}

// over

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
    type IterKind<'a, V, M, P>
        = NodeDepthVal<NodeValueData>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type QueueElement<V>
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
    type IterKind<'a, V, M, P>
        = NodeDepthVal<NodeValueNode>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy<V> + 'a,
        P: PinnedVec<N<V>> + 'a;

    type QueueElement<V>
        = (usize, NodePtr<V>)
    where
        V: TreeVariant;
}
