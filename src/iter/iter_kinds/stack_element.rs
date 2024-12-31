use crate::TreeVariant;
use orx_selfref_col::NodePtr;

pub trait StackElement<V: TreeVariant> {
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self;

    fn node_ptr(&self) -> &NodePtr<V>;
}

impl<V: TreeVariant> StackElement<V> for NodePtr<V> {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        root_ptr
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        self
    }
}

impl<V: TreeVariant> StackElement<V> for (usize, NodePtr<V>) {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        (0, root_ptr)
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.1
    }
}

impl<V: TreeVariant> StackElement<V> for (usize, usize, NodePtr<V>) {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        (0, 0, root_ptr)
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.2
    }
}
