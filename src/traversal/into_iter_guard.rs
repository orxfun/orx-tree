use crate::helpers::Col;
use crate::memory::MemoryPolicy;
use crate::pinned_storage::PinnedStorage;
use crate::traversal::enumerations::Val;
use crate::traversal::post_order::iter_ptr::PostOrderIterPtr;
use crate::TreeVariant;
use orx_selfref_col::{NodePtr, Refs};

pub struct IntoIterGuard<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    node_ptr: NodePtr<V>,
    col: *const Col<V, M, P>,
    is_root: bool,
}

impl<V, M, P> IntoIterGuard<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    /// Immediately breaks the node that `node_ptr` is pointing to from its parent:
    ///
    /// * removes the node from its parent's children references, and
    /// * clears the node's parent information.
    ///
    /// This makes the node unreachable from the root.
    ///
    /// On drop, it removes this node and all its descendants completely from the tree.
    ///
    /// # Safety
    ///
    /// This method mutates the collection and links to break the node from the tree.
    /// The caller is responsible to ensure that there will be no other mutation at this
    /// moment.
    ///
    /// The safe usage is through into_iter iterators of traversals which are created by
    /// a mutable reference of the tree, and hence, ensure that the guard creation and
    /// destruction do not overlap with any other mutation.
    pub unsafe fn new(col: &Col<V, M, P>, node_ptr: NodePtr<V>) -> Self {
        let node = unsafe { &mut *node_ptr.ptr_mut() };

        let is_root = match node.prev().get() {
            Some(parent) => {
                let parent = unsafe { &mut *parent.ptr_mut() };
                let sibling_idx = parent.next_mut().remove(node_ptr.ptr() as usize);
                debug_assert!(sibling_idx.is_some());

                node.prev_mut().clear();

                false
            }
            None => true,
        };

        Self {
            col,
            node_ptr,
            is_root,
        }
    }
}

impl<V, M, P> Drop for IntoIterGuard<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
    fn drop(&mut self) {
        let col_ptr = self.col as *mut Col<V, M, P>;
        let col = unsafe { &mut *col_ptr };

        // TODO: we have the option to choose any traversal here; they are all safe
        // with SelfRefCol. We can pick the fastest one after benchmarks.
        let iter = PostOrderIterPtr::<_, Val>::from((Default::default(), self.node_ptr.clone()));
        for ptr in iter {
            col.close_if_active(&ptr);
        }

        // the tree will be empty
        if self.is_root {
            col.ends_mut().clear();
        }
    }
}
