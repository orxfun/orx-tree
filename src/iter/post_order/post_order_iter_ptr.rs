use super::{
    depth_nodes::{DepthNode, DepthNodes},
    element::PostOrderElement,
};
use crate::{iter::Enumerator, TreeVariant};
use core::marker::PhantomData;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

/// Iterator for post order traversal
/// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
///
/// Yields node pointers; i.e., `NodePtr<V>` pointing to the traversed nodes.
pub struct PostOrderIterPtr<V, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
{
    pub(super) depth_nodes: D,
    pub(super) depth: usize,
    phantom: PhantomData<(V, E)>,
}

// impl<V, E> PostOrderIterPtr<V, DepthNodes<V>, E>
// where
//     V: TreeVariant,
//     E: PostOrderElement,
// {
//     pub(crate) fn new(root_ptr: NodePtr<V>) -> Self {
//         let mut depth_nodes = DepthNodes::default();
//         depth_nodes.init(root_ptr);
//         Self {
//             depth_nodes,
//             depth: 0,
//             phantom: PhantomData,
//         }
//     }
// }

// impl<'a, V, E> PostOrderIterPtr<V, &'a mut DepthNodes<V>, E>
// where
//     V: TreeVariant,
//     E: PostOrderElement,
// {
//     pub(crate) fn new_using(root_ptr: NodePtr<V>, depth_nodes: &'a mut DepthNodes<V>) -> Self {
//         depth_nodes.init(root_ptr);
//         Self {
//             depth_nodes,
//             depth: 0,
//             phantom: PhantomData,
//         }
//     }
// }

// iterator

impl<V, D, E> PostOrderIterPtr<V, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
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

impl<V, D, E> Iterator for PostOrderIterPtr<V, D, E>
where
    V: TreeVariant,
    D: SoM<DepthNodes<V>>,
    E: PostOrderElement,
{
    type Item = <E::Enumerator as Enumerator>::Output<NodePtr<V>>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current() {
                None => return None,
                Some(current) => match current.child() {
                    Some(child) => self.move_deeper(child),
                    _ => {
                        let ptr = current.ptr();
                        let output =
                            Some(E::element_ptr(ptr, self.depth, self.depth_nodes.get_ref()));
                        self.move_shallower();
                        return output;
                    }
                },
            }
        }
    }
}
