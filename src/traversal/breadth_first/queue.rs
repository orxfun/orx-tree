use crate::{traversal::enumeration::Enumeration, Dyn, TreeVariant};
use alloc::collections::VecDeque;
use orx_selfref_col::NodePtr;

pub type Item<V, E> = <E as Enumeration>::Item<NodePtr<V>>;

pub struct Queue<E: Enumeration> {
    queue: VecDeque<Item<Dyn<i32>, E>>,
}

impl<E: Enumeration> Queue<E> {
    pub(crate) fn for_variant<V>(&mut self) -> &mut VecDeque<Item<V, E>>
    where
        V: TreeVariant,
    {
        // # SAFETY: Size and layout of stored elements in the queue
        // solely depend on the enumeration:
        // * Val => NodePtr<V>
        // * DepthVal => (usize, NodePtr<V>)
        // * SiblingIdxVal => (usize, NodePtr<V>)
        // * DepthSiblingIdxVal => (usize, usize, NodePtr<V>)
        //
        // Since NodePtr<V> under the hood contains only one raw pointer,
        // memory size and layout of elements are independent of V.
        unsafe { core::mem::transmute(&mut self.queue) }
    }
}

impl<E: Enumeration> Default for Queue<E> {
    fn default() -> Self {
        Self {
            queue: Default::default(),
        }
    }
}
