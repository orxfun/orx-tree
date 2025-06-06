use crate::TreeVariant;
use orx_selfref_col::NodePtr;

pub struct NodePtrCon<V: TreeVariant>(pub(crate) NodePtr<V>);

impl<V: TreeVariant> Clone for NodePtrCon<V> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

unsafe impl<V: TreeVariant> Send for NodePtrCon<V> where V::Item: Send {}

unsafe impl<V: TreeVariant> Sync for NodePtrCon<V> where V::Item: Sync {}
