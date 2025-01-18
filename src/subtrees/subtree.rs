use crate::TreeVariant;

pub(crate) mod sealed {

    use crate::{
        pinned_storage::PinnedStorage, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation,
        TreeVariant,
    };
    use orx_selfref_col::NodePtr;

    pub trait SubTreeCore<Vs: TreeVariant>: Sized {
        fn root_ptr(&self) -> NodePtr<Vs> {
            todo!()
        }

        fn append_to_node_as_child<V, M, P, MO>(
            self,
            parent: &mut NodeMut<V, M, P, MO>,
            child_idx: usize,
        ) -> NodeIdx<V>
        where
            V: TreeVariant<Item = Vs::Item>,
            M: MemoryPolicy,
            P: PinnedStorage,
            MO: NodeMutOrientation;
    }
}

/// A subtree is a subset of a tree, also having a single root and satisfying structural tree properties.
///
/// SubTree implementations are used to efficiently and conveniently move parts of the tree between different trees.
pub trait SubTree<Vs>: sealed::SubTreeCore<Vs>
where
    Vs: TreeVariant,
{
}

impl<Vs, S> SubTree<Vs> for S
where
    Vs: TreeVariant,
    S: sealed::SubTreeCore<Vs>,
{
}
