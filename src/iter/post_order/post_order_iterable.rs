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
    pub fn iter<'a>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> PostIterOf<'a, V, OverData, M, P>
    where
        V: 'a,
    {
        PostOrderIter::new_using(root.col(), root.node_ptr().clone(), &mut self.depth_nodes)
    }

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

    pub fn iter_over<'a, O: IterOver>(
        &'a mut self,
        root: &'a impl NodeRef<'a, V, M, P>,
    ) -> PostIterOf<'a, V, O, M, P>
    where
        V: 'a,
    {
        PostOrderIter::new_using(root.col(), root.node_ptr().clone(), &mut self.depth_nodes)
    }

    pub fn iter_over_mut<'a, O: IterMutOver>(
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
