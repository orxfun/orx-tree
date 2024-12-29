use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

pub trait NodeRefCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn node_ptr(&self) -> &NodePtr<V>;

    fn node(&self) -> &N<V> {
        unsafe { &*self.node_ptr().ptr() }
    }
}

impl<V, M, P, X> NodeRef<V, M, P> for X
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    X: NodeRefCore<V, M, P>,
{
}

/// Reference to a tree node.
pub trait NodeRef<V, M, P>: NodeRefCore<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Returns a reference to the data of the node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// assert_eq!(root.data(), &0);
    ///
    /// let a = root.push(1);
    /// let node = tree.node(&a).unwrap();
    /// assert_eq!(node.data(), &1);
    /// ```
    #[inline(always)]
    #[allow(clippy::missing_panics_doc)]
    fn data<'a>(&'a self) -> &'a V::Item
    where
        V: 'a,
    {
        self.node()
            .data()
            .expect("node holding a tree reference must be active")
    }

    /// Returns the number of child nodes of this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let mut tree = DynTree::<i32>::new(0);
    ///
    /// let mut root = tree.root_mut().unwrap();
    /// assert_eq!(root.num_children(), 0);
    ///
    /// let a = root.push(1);
    /// let b = root.push(2);
    /// assert_eq!(root.num_children(), 2);
    ///
    /// let mut node = tree.node_mut(&a).unwrap();
    /// node.extend([3, 4, 5, 6]);
    /// assert_eq!(node.num_children(), 4);
    ///
    /// assert_eq!(tree.node(&b).unwrap().num_children(), 0);
    /// ```
    fn num_children(&self) -> usize {
        self.node().next().num_children()
    }
}
