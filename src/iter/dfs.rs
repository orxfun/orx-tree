use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    tree_variant::RefsChildren,
    Node, TreeVariant,
};
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub struct DfsNodes<'a, C, V, M = DefaultMemory<V>, P = DefaultPinVec<V>>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    C: ExactSizeIterator<Item = NodePtr<V>>,
{
    col: &'a SelfRefCol<V, M, P>,
    current_ptr: NodePtr<V>,
    depth: usize,
    position_at_depth: Vec<usize>,
    children_ptr: C,
}

impl<'a, C, V, M, P> DfsNodes<'a, C, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    C: ExactSizeIterator<Item = NodePtr<V>>,
{
    // fn new(col: &'a SelfRefCol<V, M, P>, root_ptr: NodePtr<V>) -> Self {
    //     let children_ptr = unsafe { root_ptr.node() }.next().children_ptr().cloned();
    //     Self {
    //         col,
    //         current_ptr: root_ptr,
    //         depth: 0,
    //         position_at_depth: Vec::new(),
    //         children_ptr,
    //     }
    // }
}

impl<'a, C, V, M, P> Iterator for DfsNodes<'a, C, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    C: ExactSizeIterator<Item = NodePtr<V>>,
{
    type Item = Node<'a, V, M, P>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.children_ptr.next() {
            Some(child_ptr) => {
                // asdf
                todo!()
            }
            None => {
                // no more children
                todo!()
            }
        }
    }
}
