use super::depth_nodes::{DepthNode, DepthNodes};
use super::PostOrderKind;
use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::{helpers::N, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// Iterator for post order traversal
/// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
pub struct PostOrderIter<'a, K, V, M = DefaultMemory<V>, P = DefaultPinVec<V>, D = DepthNodes<V>>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    pub(super) col: &'a SelfRefCol<V, M, P>,
    pub(super) depth_nodes: D,
    pub(super) depth: usize,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P> PostOrderIter<'a, K, V, M, P, &'a mut DepthNodes<V>>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new_using(
        col: &'a SelfRefCol<V, M, P>,
        root_ptr: NodePtr<V>,
        depth_nodes: &'a mut DepthNodes<V>,
    ) -> Self {
        depth_nodes.init(root_ptr);
        Self {
            col,
            depth_nodes,
            depth: 0,
            phantom: PhantomData,
        }
    }
}

// iterator

impl<'a, K, V, M, P, D> PostOrderIter<'a, K, V, M, P, D>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    pub(super) fn current(&self) -> Option<&DepthNode<V>> {
        match self.depth < usize::MAX {
            true => Some(self.depth_nodes.get_ref().get(self.depth)),
            false => None,
        }
    }

    pub(super) fn move_deeper(&mut self, child: NodePtr<V>) {
        let nodes = self.depth_nodes.get_mut();
        self.depth += 1;
        nodes.set(self.depth, child);
    }

    pub(super) fn move_shallower(&mut self) {
        match self.depth {
            0 => self.depth = usize::MAX,
            _ => {
                self.depth -= 1;
                self.depth_nodes.get_mut().increment_child_idx(self.depth);
            }
        }
    }
}

impl<'a, K, V, M, P, D> Iterator for PostOrderIter<'a, K, V, M, P, D>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    type Item = K::YieldElement;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current() {
                None => return None,
                Some(current) => match current.child() {
                    Some(child) => self.move_deeper(child),
                    _ => {
                        let ptr = current.ptr();
                        let x = K::element(self.col, ptr, self.depth, self.depth_nodes.get_ref());
                        self.move_shallower();
                        return Some(x);
                    }
                },
            }
        }
    }
}
