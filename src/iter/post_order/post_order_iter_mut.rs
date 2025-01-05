use super::depth_nodes::DepthNodes;
use super::{PostOrderIter, PostOrderKind};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::MemoryPolicy;

/// Mutable iterator for post order traversal
/// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
pub struct PostOrderIterMut<'a, K, V, M, P, D = DepthNodes<V>>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    iter: PostOrderIter<'a, K, V, M, P, D>,
}

// new

impl<'a, K, V, M, P, D> From<PostOrderIter<'a, K, V, M, P, D>>
    for PostOrderIterMut<'a, K, V, M, P, D>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    fn from(iter: PostOrderIter<'a, K, V, M, P, D>) -> Self {
        Self { iter }
    }
}

// iterator

impl<'a, K, V, M, P, D> Iterator for PostOrderIterMut<'a, K, V, M, P, D>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    D: SoM<DepthNodes<V>>,
{
    type Item = K::YieldElementMut;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.current() {
                None => return None,
                Some(current) => match current.child() {
                    Some(child) => self.iter.move_deeper(child),
                    _ => {
                        let ptr = current.ptr();
                        let x = K::element_mut(
                            self.iter.col,
                            ptr,
                            self.iter.depth,
                            self.iter.depth_nodes.get_ref(),
                        );
                        self.iter.move_shallower();
                        return Some(x);
                    }
                },
            }
        }
    }
}
