use crate::{Dfs, MemoryPolicy, NodeRef, Tree, TreeVariant, pinned_storage::PinnedStorage};

impl<V1, M1, P1, V2, M2, P2> PartialEq<Tree<V1, M1, P1>> for Tree<V2, M2, P2>
where
    V1: TreeVariant,
    M1: MemoryPolicy,
    P1: PinnedStorage,
    V2: TreeVariant<Item = V1::Item>,
    M2: MemoryPolicy,
    P2: PinnedStorage,
    V1::Item: PartialEq,
{
    fn eq(&self, other: &Tree<V1, M1, P1>) -> bool {
        // TODO: currently we use one of the default traversals.
        // equality check performance can be improved by walking two trees at once.
        match self.len() == other.len() {
            true => match self.len() {
                0 => true,
                _ => self
                    .root()
                    .walk::<Dfs>()
                    .zip(other.root().walk::<Dfs>())
                    .all(|(a, b)| a == b),
            },
            false => false,
        }
    }
}
