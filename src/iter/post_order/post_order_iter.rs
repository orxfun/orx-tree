use super::depth_nodes::{DepthNode, DepthNodes};
use crate::iter::IterOver;
use crate::{helpers::N, TreeVariant};
use core::marker::PhantomData;
use core::usize;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub struct PostOrderIter<'a, K, V, M, P, D>
where
    K: IterOver,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    col: &'a SelfRefCol<V, M, P>,
    depth_nodes: D,
    depth: usize,
    phantom: PhantomData<K>,
}

impl<'a, K, V, M, P, D> PostOrderIter<'a, K, V, M, P, D>
where
    K: IterOver,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    fn current(&self) -> Option<&DepthNode<V>> {
        match self.depth < usize::MAX {
            true => Some(self.depth_nodes.get_ref().get(self.depth)),
            false => None,
        }
    }

    fn move_deeper(&mut self, child: NodePtr<V>) {
        let nodes = self.depth_nodes.get_mut();
        self.depth += 1;
        nodes.set(self.depth, child);
    }

    fn move_shallower(&mut self) {
        match self.depth {
            0 => self.depth = usize::MAX,
            _ => {
                self.depth -= 1;
                self.depth_nodes.get_mut().increment_child_idx(self.depth);
            }
        }
    }

    fn current_sibling_idx(&self) -> usize {
        match self.depth {
            0 => 0,
            d => self.depth_nodes.get_ref().get(d - 1).child_idx(),
        }
    }
}

impl<'a, K, V, M, P, D> Iterator for PostOrderIter<'a, K, V, M, P, D>
where
    K: IterOver,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current() {
                None => return None,
                Some(current) => match current.child() {
                    Some(child) => {
                        self.move_deeper(child);
                    }
                    _ => {
                        let ptr = current.ptr();

                        let depth = self.depth;
                        let sibling_idx = self.current_sibling_idx();

                        self.move_shallower();
                        todo!()
                    }
                },
            }
        }
    }
}
