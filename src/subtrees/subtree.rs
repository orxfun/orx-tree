use crate::TreeVariant;

pub(crate) mod sealed {

    use crate::{
        DepthFirstSequence, MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, Tree, TreeVariant,
        pinned_storage::PinnedStorage,
    };
    use orx_selfref_col::NodePtr;

    pub trait SubTreeCore<Vs: TreeVariant>: Sized {
        fn root_ptr(&self) -> NodePtr<Vs>;

        fn root_parent_ptr(&self) -> Option<NodePtr<Vs>>;

        fn root_sibling_idx(&self) -> usize;

        fn create_subtree(&mut self) -> impl IntoIterator<Item = (usize, Vs::Item)>;

        // provided methods

        fn append_to_node_as_child<V, M, P, MO>(
            mut self,
            parent: &mut NodeMut<V, M, P, MO>,
            child_position: usize,
        ) -> NodeIdx<V>
        where
            V: TreeVariant<Item = Vs::Item>,
            M: MemoryPolicy,
            P: PinnedStorage,
            MO: NodeMutOrientation,
        {
            let subtree = self.create_subtree();
            parent.append_subtree_as_child(subtree, child_position)
        }

        fn into_new_tree<V2, M2, P2>(mut self) -> Tree<V2, M2, P2>
        where
            V2: TreeVariant<Item = Vs::Item>,
            M2: MemoryPolicy,
            P2: PinnedStorage,
            P2::PinnedVec<V2>: Default,
        {
            let subtree = self.create_subtree();
            let dfs = DepthFirstSequence::from(subtree);
            Tree::try_from(dfs).expect("subtree is a valid depth first sequence")
        }
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
