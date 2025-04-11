use super::bfs_enumeration::BreadthFirstEnumeration;
use super::queue::Item;
use crate::TreeVariant;
use crate::tree_variant::RefsChildren;
use alloc::collections::VecDeque;
use core::marker::PhantomData;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

pub struct BfsIterPtr<V, E, S = VecDeque<Item<V, E>>>
where
    E: BreadthFirstEnumeration,
    V: TreeVariant,
    S: SoM<VecDeque<Item<V, E>>>,
{
    stack: S,
    phantom: PhantomData<(V, E)>,
}

impl<V, E, S> From<(S, NodePtr<V>)> for BfsIterPtr<V, E, S>
where
    E: BreadthFirstEnumeration,
    V: TreeVariant,
    S: SoM<VecDeque<Item<V, E>>>,
{
    fn from((mut stack, root): (S, NodePtr<V>)) -> Self {
        stack.get_mut().clear();
        stack.get_mut().push_back(E::from_root(root));
        Self {
            stack,
            phantom: PhantomData,
        }
    }
}

impl<V, E> Default for BfsIterPtr<V, E, VecDeque<Item<V, E>>>
where
    E: BreadthFirstEnumeration,
    V: TreeVariant,
{
    fn default() -> Self {
        Self {
            stack: VecDeque::default(),
            phantom: PhantomData,
        }
    }
}

impl<V, E> Clone for BfsIterPtr<V, E, VecDeque<Item<V, E>>>
where
    E: BreadthFirstEnumeration,
    V: TreeVariant,
    Item<V, E>: Clone,
{
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            phantom: PhantomData,
        }
    }
}

impl<V, E, S> Iterator for BfsIterPtr<V, E, S>
where
    E: BreadthFirstEnumeration,
    V: TreeVariant,
    S: SoM<VecDeque<Item<V, E>>>,
{
    type Item = Item<V, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.get_mut().pop_front().inspect(|element| {
            let node_ptr = E::node_value(element);
            let parent = unsafe { &*node_ptr.ptr() };
            let children_ptr = parent.next().children_ptr().cloned();
            let children = E::children(element, children_ptr);
            self.stack.get_mut().extend(children);
        })
    }
}
