use crate::{
    tree_node::TreeNode,
    variants::tree_variant::{TreeEnds, TreeVariant},
};
use orx_selfref_col::{Node, SelfRefCol};
use orx_split_vec::{Recursive, SplitVec};

pub struct Tree<'a, V, T>
where
    T: 'a,
    V: TreeVariant<'a, T>,
{
    pub(crate) col: SelfRefCol<'a, V, T, SplitVec<Node<'a, V, T>, Recursive>>,
}

impl<'a, V, T> Tree<'a, V, T>
where
    T: 'a,
    V: TreeVariant<'a, T>,
{
    // new
    pub fn new() -> Self {
        Self {
            col: SelfRefCol::new(),
        }
    }

    pub fn with_root(root: T) -> Self {
        let mut tree = Self::new();
        tree.insert_root(root);
        tree
    }

    // get
    pub fn root(&self) -> Option<TreeNode<'a, V, T>> {
        self.col.ends().root().map(TreeNode::new)
    }

    pub fn num_nodes(&self) -> usize {
        self.col.len()
    }

    pub fn is_empty(&self) -> bool {
        self.col.is_empty()
    }

    // helpers
    pub(crate) fn insert_root(&mut self, root: T) {
        debug_assert!(self.is_empty());
        self.col.move_mutate(root, |x, root| {
            let root_node = x.push_get_ref(root);
            x.set_ends(root_node);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variants::{any_ary::AnyAry, dary::Binary};

    #[test]
    fn new() {
        let tree = Tree::<AnyAry, char>::new();

        assert!(tree.is_empty());
        assert_eq!(tree.num_nodes(), 0);
        assert!(tree.root().is_none());
    }

    #[test]
    fn with_root() {
        let tree = Tree::<Binary, _>::with_root('a');

        assert!(!tree.is_empty());
        assert_eq!(tree.num_nodes(), 1);
        assert_eq!(tree.root().unwrap().value(), &'a');
    }
}
