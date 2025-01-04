use super::depth_nodes::{DepthNode, DepthNodes};
use super::element::PostOrderElement;
use super::{PostOrderIterPtr, PostOrderKind};
use crate::iter::{Enumerator, NodeData};
use crate::tree::{DefaultMemory, DefaultPinVec};
use crate::{helpers::N, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub struct PostOrderIter2<'a, V, M, P, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    col: &'a SelfRefCol<V, M, P>,
    iter: PostOrderIterPtr<V, D, E>,
}

impl<'a, V, M, P, D, E> PostOrderIter2<'a, V, M, P, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a SelfRefCol<V, M, P>, iter: PostOrderIterPtr<V, D, E>) -> Self {
        Self { col, iter }
    }
}

impl<'a, V, M, P, D, E> Iterator for PostOrderIter2<'a, V, M, P, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    type Item =
        <E::Enumerator as Enumerator>::Output<<E::NodeData as NodeData>::Value<'a, V, M, P>>;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|output_ptr| E::element(self.col, output_ptr))
    }
}

// pub struct PostOrderIter2<
//     'a,
//     E,
//     D,
//     V,
//     M = DefaultMemory<V>,
//     P = DefaultPinVec<V>,
//     S = DepthNodes<V>,
// > where
//     E: PostEnumeration<V>,
//     D: NodeData,
//     V: TreeVariant,
//     M: MemoryPolicy<V>,
//     P: PinnedVec<N<V>>,
//     S: SoM<DepthNodes<V>>,
// {
//     col: &'a SelfRefCol<V, M, P>,
//     iter: PostOrderIterPtr<V, S, E>,
//     phantom: PhantomData<D>,
// }

// impl<'a, E, D, V, M, P, S> Iterator for PostOrderIter2<'a, E, D, V, M, P, S>
// where
//     E: PostEnumeration<V>,
//     D: NodeData,
//     V: TreeVariant,
//     M: MemoryPolicy<V>,
//     P: PinnedVec<N<V>>,
//     S: SoM<DepthNodes<V>>,
// {
//     type Item = usize;

//     fn next(&mut self) -> Option<Self::Item> {
//         todo!()
//     }
//     //
// }

// impl<'a, K, V, M, P, D> Iterator for PostOrderIter2<'a, K, V, M, P, D>
// where
//     K: PostOrderKind<'a, V, M, P>,
//     V: TreeVariant,
//     M: MemoryPolicy<V>,
//     P: PinnedVec<N<V>>,
//     D: SoM<DepthNodes<V>>,
// {
//     type Item = K::YieldElement;

//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next().map(|ptr| {
//             let x = K::element(self.col, ptr, self.depth, self.depth_nodes.get_ref());
//             // abc
//             todo!()
//         })
//         // loop {
//         //     match self.current() {
//         //         None => return None,
//         //         Some(current) => match current.child() {
//         //             Some(child) => self.move_deeper(child),
//         //             _ => {
//         //                 let ptr = current.ptr();
//         //                 let x = K::element(self.col, ptr, self.depth, self.depth_nodes.get_ref());
//         //                 self.move_shallower();
//         //                 return Some(x);
//         //             }
//         //         },
//         //     }
//         // }
//     }
// }

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

// new

impl<'a, K, V, M, P> PostOrderIter<'a, K, V, M, P, DepthNodes<V>>
where
    K: PostOrderKind<'a, V, M, P>,
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a SelfRefCol<V, M, P>, root_ptr: NodePtr<V>) -> Self {
        let mut depth_nodes = DepthNodes::default();
        depth_nodes.init(root_ptr);
        Self {
            col,
            depth_nodes,
            depth: 0,
            phantom: PhantomData,
        }
    }
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
