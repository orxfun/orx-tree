use crate::{pinned_storage::PinnedStorage, MemoryPolicy, Node, NodeMut, Tree, TreeVariant};
use orx_selfref_col::{MemoryState, NodeIdxError, NodePtr};

const INVALID_IDX_ERROR: &str = "\n
NodeIdx is not valid for the given tree.
Please see the notes and examples of NodeIdx and MemoryPolicy:\n
* https://docs.rs/orx-tree/latest/orx_tree/struct.NodeIdx.html\n
* https://docs.rs/orx-tree/latest/orx_tree/trait.MemoryPolicy.html\n
\n";

/// An index associated only with the node it is created for.
///
/// * Similar to usize for an array, a `NodeIdx` provides direct constant time access to the
///   node it is created for.
///   Therefore, node indices are crucial for efficiency of certain programs.
/// * Unlike usize for an array, a `NodeIdx` is specific which provides additional safety features.
///   * A node index is specific to only one node that it is created for, it can never return another node.
///   * If we create a node index from one tree and use it on another tree, we get an error ([`OutOfBounds`]).
///   * If we create a node index for a node, then we remove this node from the tree, and then we use
///     the index, we get an error ([`RemovedNode`]).
///   * If we create a node index for a node, then the nodes of the tree are reorganized to reclaim memory,
///     we get an error ([`ReorganizedCollection`]) when we try to use the node index.
///     This error is due to an implicit operation which is undesirable.
///     However, we can conveniently avoid such errors using [`Auto`] and [`Lazy`] memory reclaim policies
///     together. Please see the notes and examples in the [`MemoryPolicy`].
///
/// [`OutOfBounds`]: crate::NodeIdxError::OutOfBounds
/// [`RemovedNode`]: crate::NodeIdxError::RemovedNode
/// [`ReorganizedCollection`]: crate::NodeIdxError::ReorganizedCollection
/// [`Auto`]: crate::Auto
/// [`Lazy`]: crate::Lazy
/// [`MemoryPolicy`]: crate::MemoryPolicy
pub struct NodeIdx<V: TreeVariant>(orx_selfref_col::NodeIdx<V>);

impl<V: TreeVariant> Clone for NodeIdx<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<V: TreeVariant> PartialEq for NodeIdx<V> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<V: TreeVariant> NodeIdx<V> {
    #[inline(always)]
    pub(crate) fn new(state: MemoryState, node_ptr: &NodePtr<V>) -> Self {
        Self(orx_selfref_col::NodeIdx::new(state, node_ptr))
    }

    #[inline(always)]
    pub fn is_valid_for<M, P>(&self, tree: &Tree<V, M, P>) -> bool
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        self.0.is_valid_for(&tree.0)
    }

    #[inline(always)]
    pub fn node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.0.is_valid_for(&tree.0), "{}", INVALID_IDX_ERROR);
        Node::new(&tree.0, self.0.node_ptr())
    }

    #[inline(always)]
    pub fn node_mut<'a, M, P>(&self, tree: &'a mut Tree<V, M, P>) -> NodeMut<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        assert!(self.0.is_valid_for(&tree.0), "{}", INVALID_IDX_ERROR);
        NodeMut::new(&mut tree.0, self.0.node_ptr())
    }

    #[inline(always)]
    pub fn get_node<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Option<Node<'a, V, M, P>>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        self.0
            .is_valid_for(&tree.0)
            .then(|| Node::new(&tree.0, self.0.node_ptr()))
    }

    #[inline(always)]
    pub fn get_node_mut<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> Option<NodeMut<'a, V, M, P>>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        self.0
            .is_valid_for(&tree.0)
            .then(|| NodeMut::new(&mut tree.0, self.0.node_ptr()))
    }

    #[inline(always)]
    pub fn try_get_node<'a, M, P>(
        &self,
        tree: &'a Tree<V, M, P>,
    ) -> Result<Node<'a, V, M, P>, NodeIdxError>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        tree.0
            .try_get_ptr(&self.0)
            .map(|ptr| Node::new(&tree.0, ptr))
    }

    #[inline(always)]
    pub fn try_get_node_mut<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> Result<NodeMut<'a, V, M, P>, NodeIdxError>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        tree.0
            .try_get_ptr(&self.0)
            .map(|ptr| NodeMut::new(&mut tree.0, ptr))
    }

    #[inline(always)]
    pub unsafe fn node_unchecked<'a, M, P>(&self, tree: &'a Tree<V, M, P>) -> Node<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        Node::new(&tree.0, self.0.node_ptr())
    }

    #[inline(always)]
    pub unsafe fn node_mut_unchecked<'a, M, P>(
        &self,
        tree: &'a mut Tree<V, M, P>,
    ) -> NodeMut<'a, V, M, P>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
    {
        NodeMut::new(&mut tree.0, self.0.node_ptr())
    }
}

#[test]
fn abc() {
    use crate::*;
    use alloc::vec::Vec;

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11

    let mut tree = DynTree::<i32>::new(1);

    let mut root = tree.root_mut().unwrap();
    let [id2, id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree).push(8);

    let mut n3 = id3.node_mut(&mut tree);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree).push(9);
    id7.node_mut(&mut tree).extend([10, 11]);

    // traversal from any node

    let root = tree.root().unwrap();
    let values: Vec<_> = root.dfs().copied().collect();
    assert_eq!(values, [1, 2, 4, 8, 5, 3, 6, 9, 7, 10, 11]);

    let n3 = id3.node(&tree);
    let values: Vec<_> = n3.dfs().copied().collect();
    assert_eq!(values, [3, 6, 9, 7, 10, 11]);

    let n7 = id7.node(&tree);
    let values: Vec<_> = n7.dfs().copied().collect();
    assert_eq!(values, [7, 10, 11]);
}
