use super::depth_nodes::DepthNodes;
use super::{PostOrderElement, PostOrderIter, PostOrderIterPtr, PostOrderKind};
use crate::iter::{Enumerator, NodeData};
use crate::{helpers::N, TreeVariant};
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, SelfRefCol};

pub struct PostOrderIterMut2<'a, V, M, P, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    col: &'a mut SelfRefCol<V, M, P>,
    iter: PostOrderIterPtr<V, D, E>,
}

impl<'a, V, M, P, D, E> PostOrderIterMut2<'a, V, M, P, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a mut SelfRefCol<V, M, P>, iter: PostOrderIterPtr<V, D, E>) -> Self {
        Self { col, iter }
    }
}

// impl<'a, V, M, P, D, E> Iterator for PostOrderIterMut2<'a, V, M, P, D, E>
// where
//     V: TreeVariant + 'a,
//     D: SoM<DepthNodes<V>> + 'a,
//     E: PostOrderElement + 'a,
//     M: MemoryPolicy<V> + 'a,
//     P: PinnedVec<N<V>> + 'a,
// {
//     type Item =
//         <E::Enumerator as Enumerator>::Output<<E::NodeData as NodeData>::ValueMut<'a, V, M, P>>;

//     #[inline(always)]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter
//             .next()
//             .map(move |output_ptr| E::element_mut(self.col, output_ptr))
//     }
// }

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
