use crate::TreeVariant;

pub(crate) mod sealed {

    use crate::{
        MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant,
        pinned_storage::PinnedStorage,
    };

    pub trait SubTreeWithinCore<V: TreeVariant>: Sized {
        fn append_to_node_within_as_child<M, P, MO>(
            self,
            parent: &mut NodeMut<V, M, P, MO>,
            child_position: usize,
        ) -> NodeIdx<V>
        where
            M: MemoryPolicy,
            P: PinnedStorage,
            MO: NodeMutOrientation;
    }
}

/// A subtree is a subset of a tree, also having a single root and satisfying structural tree properties.
///
/// SubTreeWithin implementations are used to efficiently and conveniently move parts of the tree within itself.
pub trait SubTreeWithin<V: TreeVariant>: sealed::SubTreeWithinCore<V> {}

impl<V: TreeVariant, S: sealed::SubTreeWithinCore<V>> SubTreeWithin<V> for S {}
