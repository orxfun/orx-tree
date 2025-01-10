use crate::{
    memory::MemoryPolicy, pinned_storage::PinnedStorage, Node, NodeMut, NodeRef, TreeVariant,
};
use core::fmt::Debug;

impl<V, M, P> Debug for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt_node(f, self)
    }
}

impl<V, M, P> Debug for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt_node(f, self)
    }
}

fn fmt_node<'a, V, M, P>(
    f: &mut core::fmt::Formatter<'_>,
    node: &impl NodeRef<'a, V, M, P>,
) -> core::fmt::Result
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    // TODO: additional info, such as depth
    f.debug_struct("Node")
        .field("is_root", &node.is_root())
        .field("sibling_position", &node.sibling_position())
        .field("num_children", &node.num_children())
        .field("data", node.data())
        .finish()
}
