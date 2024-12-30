use crate::{
    helpers::N,
    tree::{DefaultMemory, DefaultPinVec},
    tree_variant::RefsChildren,
    Node, TreeVariant,
};
use alloc::vec::Vec;
use orx_pinned_vec::PinnedVec;
use orx_self_or::SoM;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

/// # Safety
///
/// It is safe to use this method within this file which guarantees that:
/// * node_ptr points to an active node with a valid pointer.
/// * created node reference is not leaked by the next method of the iterator.
///
/// node_ptr points to a node belonging to the collection `col`.
/// The collection is referenced for the lifetime of 'a; therefore, the
/// node that the pointer is pointing at will be valid for at least 'a.
fn node<'a, V: TreeVariant>(node_ptr: &NodePtr<V>) -> &'a N<V> {
    unsafe { &*node_ptr.ptr() }
}

/// A depth first search iterator; also known as "pre-order traversal" ([wiki](https://en.wikipedia.org/wiki/Tree_traversal#Pre-order_implementation)).
pub struct DfsNodes<'a, V, M = DefaultMemory<V>, P = DefaultPinVec<V>, S = Vec<NodePtr<V>>>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
    S: SoM<Vec<NodePtr<V>>>,
{
    col: &'a SelfRefCol<V, M, P>,
    stack: S,
}

impl<'a, V, M, P> DfsNodes<'a, V, M, P, Vec<NodePtr<V>>>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(col: &'a SelfRefCol<V, M, P>, root_ptr: NodePtr<V>) -> Self {
        let mut stack = Vec::new();
        stack.push(root_ptr);
        Self { col, stack }
    }
}

impl<'a, V, M, P> DfsNodes<'a, V, M, P, &'a mut Vec<NodePtr<V>>>
where
    V: TreeVariant,
    M: MemoryPolicy<V>,
    P: PinnedVec<N<V>>,
{
    pub(crate) fn new(
        col: &'a SelfRefCol<V, M, P>,
        root_ptr: NodePtr<V>,
        stack: &'a mut Vec<NodePtr<V>>,
    ) -> Self {
        stack.get_mut().clear();
        stack.get_mut().push(root_ptr);
        Self { col, stack }
    }
}

impl<'a, V, M, P, S> Iterator for DfsNodes<'a, V, M, P, S>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    S: SoM<Vec<NodePtr<V>>>,
{
    type Item = Node<'a, V, M, P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.get_mut().pop().map(|node_ptr| {
            let node = node(&node_ptr);
            self.stack
                .get_mut()
                .extend(node.next().children_ptr().cloned());
            Node::new(self.col, node_ptr)
        })
    }
}

#[test]
fn abc() {
    //
}
