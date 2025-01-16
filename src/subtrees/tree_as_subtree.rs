use super::subtree::SubTree;
use crate::{pinned_storage::PinnedStorage, MemoryPolicy, Tree, TreeVariant};

impl<V, M, P> SubTree<V::Item> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
{
}
