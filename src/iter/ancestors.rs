use crate::{helpers::Col, pinned_storage::PinnedStorage, MemoryPolicy, Node, TreeVariant};
use orx_selfref_col::NodePtr;

/// Ancestors iterators which starts from a node and yields nodes bottom-up until the root
/// of the tree is reached.
pub struct AncestorsIter<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    col: &'a Col<V, M, P>,
    current: Option<NodePtr<V>>,
}

impl<'a, V, M, P> AncestorsIter<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    pub(crate) fn new(col: &'a Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        Self {
            col,
            current: Some(node_ptr),
        }
    }
}

impl<'a, V, M, P> Iterator for AncestorsIter<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    type Item = Node<'a, V, M, P>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.clone().map(|ptr| {
            let node = unsafe { &*ptr.ptr() };
            self.current = node.prev().get().cloned();
            Node::new(self.col, ptr)
        })
    }
}
