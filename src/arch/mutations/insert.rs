use crate::{tree::Tree, tree_node::TreeNode, variants::tree_variant::TreeVariant};
use std::marker::PhantomData;

pub enum Insertion<'a, V, T>
where
    T: 'a,
    V: TreeVariant<'a, T>,
{
    None,
    AsParentOf(TreeNode<'a, V, T>),
    AsChildOf(TreeNode<'a, V, T>, usize),
}

pub struct Insert<'a, V, T, Fun>
where
    T: 'a,
    V: TreeVariant<'a, T>,
    Fun: Fn(TreeNode<'a, V, T>) -> Insertion<'a, V, T>,
{
    fun: Fun,
    phantom: PhantomData<&'a (V, T)>,
}

impl<'a, V, T> Tree<'a, V, T>
where
    T: 'a,
    V: TreeVariant<'a, T>,
{
    pub fn insert<Fun>(&mut self, search_insertion: Fun, value: T)
    where
        Fun: Fn(TreeNode<'a, V, T>) -> Insertion<'a, V, T>,
    {
        match self.root() {
            None => self.insert_root(value),
            Some(root) => {
                let insertion = search_insertion(root);
                V::insert(self, insertion);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variants::dary::Binary;

    #[test]
    fn insert_root() {
        let mut tree: Tree<Binary, _> = Tree::new();
        assert!(tree.is_empty());

        tree.insert(|_| Insertion::None, 42);
        assert_eq!(tree.num_nodes(), 1);
        assert_eq!(tree.root().unwrap().value(), &42);
    }
}
