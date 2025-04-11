use crate::{Dyn, TreeVariant, traversal::enumeration::Enumeration};
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

pub type Item<V, E> = <E as Enumeration>::Item<NodePtr<V>>;

pub struct Stack<E: Enumeration> {
    stack: Vec<Item<Dyn<i32>, E>>,
}

impl<E: Enumeration> Stack<E> {
    pub(crate) fn for_variant<V>(&mut self) -> &mut Vec<Item<V, E>>
    where
        V: TreeVariant,
    {
        // # SAFETY: Size and layout of stored elements in the stack
        // solely depend on the enumeration:
        // * Val => NodePtr<V>
        // * DepthVal => (usize, NodePtr<V>)
        // * SiblingIdxVal => (usize, NodePtr<V>)
        // * DepthSiblingIdxVal => (usize, usize, NodePtr<V>)
        //
        // Since NodePtr<V> under the hood contains only one raw pointer,
        // memory size and layout of elements are independent of V.
        unsafe { core::mem::transmute(&mut self.stack) }
    }
}

impl<E: Enumeration> Default for Stack<E> {
    fn default() -> Self {
        Self {
            stack: Default::default(),
        }
    }
}
