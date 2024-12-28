use crate::{
    helpers::N,
    tree_col::{TreeColCore, TreeColMutCore},
    Node, NodeMut, TreeVariant,
};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{
    MemoryPolicy, MemoryReclaimOnThreshold, NodeIdx, NodePtr, RefsSingle, SelfRefCol,
};
use orx_split_vec::{Recursive, SplitVec};

#[allow(type_alias_bounds)]
pub(crate) type DefaultMemory<V: TreeVariant> = MemoryReclaimOnThreshold<2, V, V::Reclaimer>;

pub(crate) type DefaultPinVec<V> = SplitVec<N<V>, Recursive>;

/// Core tree structure.
pub struct Tree<V, M = DefaultMemory<V>, P = DefaultPinVec<V>>(pub(crate) SelfRefCol<V, M, P>)
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>;

impl<V, M, P> TreeColCore<V, M, P> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn col(&self) -> &SelfRefCol<V, M, P> {
        &self.0
    }
}

impl<V, M, P> TreeColMutCore<V, M, P> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    #[inline(always)]
    fn col_mut(&mut self) -> &mut SelfRefCol<V, M, P> {
        &mut self.0
    }
}

impl<V, M, P> Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Creates an empty tree.
    ///
    /// You may call [`push_root`] to instantiate the empty tree.
    ///
    /// [`push_root`]: Self::push_root
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_tree::*;
    ///
    /// let tree: DynTree<i32> = DynTree::empty();
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.root(), None);
    /// ```
    pub fn empty() -> Self
    where
        P: Default,
    {
        Self(SelfRefCol::new())
    }

    /// Creates a new tree including the root node with the given `root_value`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use orx_tree::*;
    ///
    /// let tree: DynTree<i32> = DynTree::new(42);
    ///
    /// assert_eq!(tree.len(), 1);
    /// assert_eq!(tree.root().unwrap().data(), &42);
    /// ```
    pub fn new(root_value: V::Item) -> Self
    where
        P: Default,
    {
        let mut col = SelfRefCol::<V, M, P>::new();
        let root_ptr = col.push(root_value);
        let root_mut: &mut RefsSingle<V> = col.ends_mut();
        root_mut.set_some(root_ptr);

        Self(col)
    }

    /// ***O(1)*** Returns the number of nodes in the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree: DynTree<i32> = DynTree::new(42);
    /// assert_eq!(tree.len(), 1);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// let _ = root.push(4);
    /// let idx = root.push(2);
    /// assert_eq!(tree.len(), 3);
    ///
    /// let mut node = tree.node_mut(&idx).unwrap();
    /// node.push(7);
    /// assert_eq!(tree.len(), 4);
    /// ```
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the tree is empty.
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Pushes the root to the empty tree.
    ///
    /// # Panics
    ///
    /// Panics if push_root is called when the tree is not empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree: DynTree<i32> = DynTree::empty();
    ///
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.root(), None);
    ///
    /// tree.push_root(42);
    /// assert!(!tree.is_empty());
    /// assert_eq!(tree.len(), 1);
    /// assert_eq!(tree.root().unwrap().data(), &42);
    /// ```
    pub fn push_root(&mut self, root_value: V::Item) -> NodeIdx<V> {
        assert!(
            self.is_empty(),
            "Cannot push root to the tree which already has a root."
        );

        let root_idx = self.0.push_get_idx(root_value);
        let root_mut: &mut RefsSingle<V> = self.0.ends_mut();
        root_mut.set_some(root_idx.node_ptr());

        root_idx
    }

    /// Removes all the nodes including the root of the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree: BinaryTree<i32> = BinaryTree::new(42);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// root.push(4);
    /// let idx = root.push(2);
    ///
    /// let mut node = tree.node_mut(&idx).unwrap();
    /// node.push(7);
    ///
    /// assert_eq!(tree.len(), 4);
    /// assert_eq!(tree.root().unwrap().data(), &42);
    ///
    /// tree.clear();
    /// assert!(tree.is_empty());
    /// assert_eq!(tree.root(), None);
    /// ```
    pub fn clear(&mut self) {
        self.0.clear();
        self.0.ends_mut().set_none();
    }

    // get nodes

    /// Returns the root node of the tree; None if the tree is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// // initiate a rooted tree
    /// let mut tree = DynTree::<_>::new('a');
    /// assert_eq!(tree.root().unwrap().data(), &'a');
    ///
    /// tree.clear();
    /// assert_eq!(tree.root(), None);
    ///
    /// // initiate an empty tree
    /// let mut tree = BinaryTree::<_>::empty();
    /// assert_eq!(tree.root(), None);
    ///
    /// tree.push_root('a');
    /// assert_eq!(tree.root().unwrap().data(), &'a');
    /// ```
    pub fn root(&self) -> Option<Node<V, M, P>> {
        self.root_ptr().cloned().map(|p| self.ptr_to_tree_node(p))
    }

    /// Returns the root as a mutable node of the tree; None if the tree is empty.
    pub fn root_mut(&mut self) -> Option<NodeMut<V, M, P>> {
        self.root_ptr()
            .cloned()
            .map(|p| self.ptr_to_tree_node_mut(p))
    }

    /// Returns the node with the given `node_idx`; returns None if the index is invalid.
    pub fn node(&self, node_idx: &NodeIdx<V>) -> Option<Node<V, M, P>> {
        self.0.get_ptr(node_idx).map(|p| self.ptr_to_tree_node(p))
    }

    /// Returns the mutable node with the given `node_idx`; returns None if the index is invalid.
    pub fn node_mut(&mut self, node_idx: &NodeIdx<V>) -> Option<NodeMut<V, M, P>> {
        self.0
            .get_ptr(node_idx)
            .map(|p| self.ptr_to_tree_node_mut(p))
    }

    // helpers

    /// Returns the pointer to the root; None if empty.
    fn root_ptr(&self) -> Option<&NodePtr<V>> {
        self.0.ends().get()
    }
}

#[test]
fn abc() {
    use crate::*;

    let mut tree: DynTree<i32> = DynTree::empty();

    assert!(tree.is_empty());
    assert_eq!(tree.root(), None);

    tree.push_root(42);
    assert!(!tree.is_empty());
    assert_eq!(tree.len(), 1);

    assert_eq!(tree.root().unwrap().data(), &42);
}
