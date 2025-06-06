use crate::TreeVariant;
use orx_selfref_col::NodePtr;

/// Ancestors iterators which starts from a node and yields nodes bottom-up until the root
/// of the tree is reached.
pub struct AncestorsIterPtr<V>
where
    V: TreeVariant,
{
    root_ptr: NodePtr<V>,
    current: Option<NodePtr<V>>,
}

impl<V: TreeVariant> AncestorsIterPtr<V> {
    pub(crate) fn new(root_ptr: NodePtr<V>, descendant_ptr: NodePtr<V>) -> Self {
        Self {
            root_ptr,
            current: Some(descendant_ptr),
        }
    }
}

impl<V: TreeVariant> Iterator for AncestorsIterPtr<V> {
    type Item = NodePtr<V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.clone().map(|ptr| {
            let node = unsafe { &*ptr.ptr() };

            self.current = match ptr == self.root_ptr {
                false => node.prev().get().cloned(),
                true => None,
            };

            ptr
        })
    }
}

impl<V: TreeVariant> Clone for AncestorsIterPtr<V> {
    fn clone(&self) -> Self {
        Self {
            root_ptr: self.root_ptr.clone(),
            current: self.current.clone(),
        }
    }
}

unsafe impl<V: TreeVariant> Send for AncestorsIterPtr<V> where V::Item: Send {}

unsafe impl<V: TreeVariant> Sync for AncestorsIterPtr<V> where V::Item: Sync {}
