use crate::{tree_variant::RefsChildren, TreeVariant};
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

// element

pub struct DepthNode<V: TreeVariant> {
    pointer: NodePtr<V>,
    child_idx: usize,
}

impl<V: TreeVariant> From<NodePtr<V>> for DepthNode<V> {
    fn from(pointer: NodePtr<V>) -> Self {
        DepthNode {
            pointer,
            child_idx: 0,
        }
    }
}

impl<V: TreeVariant> Clone for DepthNode<V> {
    fn clone(&self) -> Self {
        Self {
            pointer: self.pointer.clone(),
            child_idx: self.child_idx,
        }
    }
}

impl<V: TreeVariant> DepthNode<V> {
    #[inline(always)]
    pub fn child(&self) -> Option<NodePtr<V>> {
        let node = unsafe { &*self.pointer.ptr() };
        node.next().get_ptr(self.child_idx).cloned()
    }

    #[inline(always)]
    pub fn ptr(&self) -> NodePtr<V> {
        self.pointer.clone()
    }

    #[inline(always)]
    pub fn child_idx(&self) -> usize {
        self.child_idx
    }
}

// collection

/// TO BE REMOVED
pub struct DepthNodes<V: TreeVariant> {
    vec: Vec<DepthNode<V>>,
}

impl<V: TreeVariant> Default for DepthNodes<V> {
    fn default() -> Self {
        Self {
            vec: Default::default(),
        }
    }
}

impl<V: TreeVariant> DepthNodes<V> {
    /// TO BE REMOVED
    pub fn init(&mut self, root_ptr: NodePtr<V>) {
        self.vec.clear();
        self.vec.push(root_ptr.into());
    }

    /// TO BE REMOVED
    pub fn get(&self, d: usize) -> &DepthNode<V> {
        &self.vec[d]
    }

    /// TO BE REMOVED
    pub fn get_cloned(&self, d: usize) -> DepthNode<V> {
        self.vec[d].clone()
    }

    /// TO BE REMOVED
    pub fn set(&mut self, d: usize, pointer: NodePtr<V>) {
        match self.vec.get_mut(d) {
            Some(x) => *x = pointer.into(),
            None => {
                debug_assert!(self.vec.len() == d);
                self.vec.push(pointer.into());
            }
        }
    }

    /// TO BE REMOVED
    pub fn increment_child_idx(&mut self, d: usize) {
        self.vec[d].child_idx += 1;
    }
}
