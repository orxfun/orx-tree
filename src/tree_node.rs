use crate::variants::tree_variant::TreeVariant;
use orx_selfref_col::NodeRefs;

pub struct TreeNode<'a, V, T>
where
    T: 'a,
    V: TreeVariant<'a, T>,
{
    pub(crate) node: &'a orx_selfref_col::Node<'a, V, T>,
}

impl<'a, V, T> TreeNode<'a, V, T>
where
    T: 'a,
    V: TreeVariant<'a, T>,
{
    pub(crate) fn new(node: &'a orx_selfref_col::Node<'a, V, T>) -> Self {
        Self { node }
    }

    pub fn value(&self) -> &T {
        unsafe { self.node.data().unwrap_unchecked() }
    }

    pub fn parent(&self) -> Option<Self> {
        self.node.prev().get().map(Self::new)
    }

    pub fn children(&self) -> impl Iterator<Item = Self> {
        self.node.next().referenced_nodes().map(Self::new)
    }
}

#[cfg(test)]
mod tests {
    use crate::{tree::Tree, variants::dary::Binary};

    #[test]
    fn root_value() {
        let tree: Tree<Binary, _> = Tree::with_root(42);
        assert_eq!(tree.root().unwrap().value(), &42);
    }

    #[test]
    fn root_parent() {
        let tree: Tree<Binary, _> = Tree::with_root(42);
        assert!(tree.root().unwrap().parent().is_none());
    }

    #[test]
    fn root_empty_children() {
        let tree: Tree<Binary, _> = Tree::with_root(42);
        assert_eq!(tree.root().unwrap().children().count(), 0);
    }
}
