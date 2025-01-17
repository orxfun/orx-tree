pub(crate) mod sealed {
    use crate::{
        pinned_storage::PinnedStorage, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation,
        TreeVariant,
    };

    pub trait SubTreeCore<T>: Sized {
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
}

/// A subtree is a subset of a tree, also having a single root and satisfying structural tree properties.
///
/// SubTree implementations are used to efficiently and conveniently move parts of the tree within itself,
/// and move subtrees among different trees.
pub trait SubTree<T>: sealed::SubTreeCore<T> {}

impl<T, S> SubTree<T> for S where S: sealed::SubTreeCore<T> {}
