use orx_selfref_col::{
    MemoryReclaimOnThreshold, Node, NodeDataLazyClose, NodeRefSingle, NodeRefs, Variant,
};

use crate::{mutations::insert::Insertion, tree::Tree};

pub trait TreeVariant<'a, T>:
    Variant<
    'a,
    T,
    Storage = NodeDataLazyClose<T>,
    Prev = NodeRefSingle<'a, Self, T>,
    Ends = NodeRefSingle<'a, Self, T>,
    MemoryReclaim = MemoryReclaimOnThreshold<2>,
>
where
    Self: 'a,
    T: 'a,
    Self::Ends: TreeEnds<'a, Self, T>,
{
    fn insert(tree: &mut Tree<'a, Self, T>, insertion: Insertion<'a, Self, T>);
}

pub trait TreeEnds<'a, V, T>
where
    V: TreeVariant<'a, T>,
    T: 'a,
{
    fn root(&self) -> Option<&'a Node<'a, V, T>>;
}

impl<'a, V, T> TreeEnds<'a, V, T> for NodeRefSingle<'a, V, T>
where
    V: TreeVariant<'a, T>,
    T: 'a,
{
    #[inline(always)]
    fn root(&self) -> Option<&'a Node<'a, V, T>> {
        *self.get()
    }
}
