use super::subtree_within::sealed::SubTreeWithinCore;
use crate::{
    MemoryPolicy, NodeIdx, NodeMut, NodeMutOrientation, TreeVariant, iter::AncestorsIterPtr,
    node_ref::NodeRefCore, pinned_storage::PinnedStorage, tree_node_idx::INVALID_IDX_ERROR,
    tree_variant::RefsChildren,
};
use orx_selfref_col::Refs;

pub struct MovedSubTreeWithin<V: TreeVariant> {
    idx: NodeIdx<V>,
}

impl<V: TreeVariant> MovedSubTreeWithin<V> {
    pub(crate) fn new(idx: NodeIdx<V>) -> Self {
        Self { idx }
    }
}

impl<V: TreeVariant> SubTreeWithinCore<V> for MovedSubTreeWithin<V> {
    fn append_to_node_within_as_child<M, P, MO>(
        self,
        parent: &mut NodeMut<V, M, P, MO>,
        child_position: usize,
    ) -> NodeIdx<V>
    where
        M: MemoryPolicy,
        P: PinnedStorage,
        MO: NodeMutOrientation,
    {
        let root_ptr = parent.root_ptr().expect("non-empty tree");
        let ptr_parent = parent.node_ptr();
        let ptr_child = parent
            .col()
            .try_get_ptr(self.idx.0)
            .expect(INVALID_IDX_ERROR);
        assert!(
            AncestorsIterPtr::new(root_ptr, ptr_parent).all(|x| x != ptr_child),
            "Cannot move a subtree as a child/sibling of itself, or as a child/sibling of any descendant node belonging to the same subtree."
        );

        let node_child = unsafe { &mut *ptr_child.ptr_mut() };

        if let Some(ptr_old_parent) = node_child.prev().get() {
            if ptr_old_parent == ptr_parent {
                // move will lead to the same state; hence, not necessary
                return self.idx;
            }

            let old_parent = unsafe { &mut *ptr_old_parent.ptr_mut() };
            old_parent
                .next_mut()
                .remove(unsafe { ptr_child.ptr() as usize });
        }
        node_child.prev_mut().set_some(ptr_parent);

        let node_parent = unsafe { &mut *ptr_parent.ptr_mut() };
        node_parent.next_mut().insert(child_position, ptr_child);

        self.idx
    }
}
