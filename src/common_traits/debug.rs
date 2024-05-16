use crate::{tree::Tree, tree_node::TreeNode, variants::tree_variant::TreeVariant};
use std::fmt::Debug;

impl<'a, V, T> Debug for TreeNode<'a, V, T>
where
    T: 'a + Debug,
    V: TreeVariant<'a, T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeNode")
            .field("node", &self.node.data().expect("is-some"))
            .finish()
    }
}

impl<'a, V, T> Debug for Tree<'a, V, T>
where
    T: 'a + Debug,
    V: TreeVariant<'a, T>,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tree")
            .field("num_nodes", &self.num_nodes())
            .field("root", &self.root())
            .finish()
    }
}
