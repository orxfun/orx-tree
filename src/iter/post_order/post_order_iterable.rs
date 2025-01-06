use super::{depth_nodes::DepthNodes, PostOrderIter, PostOrderIterMut};
use crate::{
    helpers::N,
    iter::{IterMutOver, IterOver, OverData},
    node_ref::NodeRefCore,
    tree::{DefaultMemory, DefaultPinVec},
    NodeMut, NodeRef, TreeVariant,
};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::MemoryPolicy;

/// An iterable which can create post-order iterators over and over, using the same only-once allocated vector.
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
///
/// let mut tree = DynTree::<i32>::new(1);
///
/// let mut root = tree.root_mut().unwrap();
/// let [id2, id3] = root.grow([2, 3]);
///
/// let mut n2 = id2.node_mut(&mut tree);
/// let [id4, _] = n2.grow([4, 5]);
///
/// let [id8] = id4.node_mut(&mut tree).grow([8]);
///
/// let mut n3 = id3.node_mut(&mut tree);
/// let [id6, id7] = n3.grow([6, 7]);
///
/// id6.node_mut(&mut tree).push(9);
/// id7.node_mut(&mut tree).extend([10, 11]);
///
/// // create the iterable for post-order traversal
/// // that creates the internal vector once
///
/// let mut po = TraversalDepr::post_order();
///
/// // repeatedly create iterators from it, without allocation
///
/// let root = tree.root().unwrap();
/// let values: Vec<_> = po.iter(&root).copied().collect();
/// assert_eq!(values, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);
///
/// let mut n7 = id7.node_mut(&mut tree);
/// for (i, value) in po.iter_mut(&mut n7).enumerate() {
///     *value += (i * 100) as i32;
/// }
///
/// let n3 = id3.node(&tree);
/// let values: Vec<_> = po.iter(&n3).copied().collect();
/// assert_eq!(values, [9, 6, 10, 111, 207, 3]);
///
/// let n7 = id7.node(&tree);
/// let values: Vec<_> = po.iter(&n7).copied().collect();
/// assert_eq!(values, [10, 111, 207]);
///
/// // we may also create iterators over other elements
///
/// let root = tree.root().unwrap();
/// let mut iter = po.iter_over::<OverDepthSiblingData>(&root);
/// assert_eq!(iter.next(), Some((3, 0, &8)));
/// assert_eq!(iter.next(), Some((2, 0, &4)));
/// assert_eq!(iter.next(), Some((2, 1, &5)));
/// assert_eq!(iter.next(), Some((1, 0, &2)));
/// assert_eq!(iter.next(), Some((3, 0, &9)));
///
/// let mut iter = po.iter_over::<OverDepthNode>(&root);
///
/// let (d, node) = iter.next().unwrap();
/// assert_eq!(d, 3);
/// assert_eq!(&node.idx(), &id8);
/// assert_eq!(node.num_children(), 0);
///
/// let (d, node) = iter.next().unwrap();
/// assert_eq!(d, 2);
/// assert_eq!(&node.idx(), &id4);
/// assert_eq!(node.num_children(), 1);
/// assert_eq!(&node.child(0).unwrap().idx(), &id8);
/// ```
pub struct PostOrderIterable<
    V: TreeVariant,
    M: MemoryPolicy<V> = DefaultMemory<V>,
    P: PinnedVec<N<V>> = DefaultPinVec<V>,
> {
    depth_nodes: DepthNodes<V>,
    phantom: PhantomData<(M, P)>,
}

impl<V, M, P> Default for PostOrderIterable<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    fn default() -> Self {
        Self {
            depth_nodes: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<V, M, P> PostOrderIterable<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    /// Creates a post-order iterator for the tree rooted at the given `root` node.
    ///
    /// Item of the created iterator is a reference to the of the nodes; i.e., [`data`].
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    ///
    /// [`data`]: crate::NodeRef::data
    pub fn iter<'a>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> PostIterOf<'a, V, OverData, M, P>
    where
        V: 'a,
    {
        PostOrderIter::new_using(root.col(), root.node_ptr().clone(), &mut self.depth_nodes)
    }

    /// Creates a mutable post-order iterator for the tree rooted at the given `root` node.
    ///
    /// Item of the created iterator is a mutable reference to the of the nodes; i.e., [`data_mut`].
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    ///
    /// [`data_mut`]: crate::NodeMut::data_mut
    pub fn iter_mut<'a>(
        &'a mut self,
        root: &'a mut NodeMut<'a, V, M, P>,
    ) -> PostIterMutOf<'a, V, OverData, M, P>
    where
        V: 'a,
    {
        PostOrderIter::new_using(root.col(), root.node_ptr().clone(), &mut self.depth_nodes).into()
    }

    // over

    /// Creates a post-order iterator for the tree rooted at the given `root` node.
    ///
    /// Item of the created iterator is determined by the generic parameter `O`: [`IterOver`].
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_over<'a, O: IterOver>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> PostIterOf<'a, V, O, M, P>
    where
        V: 'a,
    {
        PostOrderIter::new_using(root.col(), root.node_ptr().clone(), &mut self.depth_nodes)
    }

    /// Creates a mutable post-order iterator for the tree rooted at the given `root` node.
    ///
    /// Note that this `root` might be an intermediate node of a larger tree.
    /// Regardless, the depth-first-search will be rooted at this node:
    /// * root's depth will be assumed to be zero,
    /// * root's sibling index will be assumed to be zero since its siblings, if any, are not relevant to the search.
    pub fn iter_mut_over<'a, O: IterMutOver>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> PostIterMutOf<'a, V, O, M, P>
    where
        V: 'a,
    {
        PostOrderIter::new_using(root.col(), root.node_ptr().clone(), &mut self.depth_nodes).into()
    }
}

// type simplification of iterators

type PostIterOf<'a, V, K, M, P> =
    PostOrderIter<'a, <K as IterOver>::PostOrderKind<'a, V, M, P>, V, M, P, &'a mut DepthNodes<V>>;

type PostIterMutOf<'a, V, K, M, P> = PostOrderIterMut<
    'a,
    <K as IterOver>::PostOrderKind<'a, V, M, P>,
    V,
    M,
    P,
    &'a mut DepthNodes<V>,
>;
