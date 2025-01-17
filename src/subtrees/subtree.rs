use crate::{
    pinned_storage::PinnedStorage, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant,
};

pub trait SubTree<T>: Sized {
    fn append_to_node_as_child<V, M, P, MO>(
        self,
        parent: &mut NodeMut<V, M, P, MO>,
        child_idx: usize,
    ) -> NodeIdx<V>
    where
        V: TreeVariant<Item = T>,
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation;
}
