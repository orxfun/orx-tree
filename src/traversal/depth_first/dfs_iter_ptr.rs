use crate::tree_variant::RefsChildren;
use crate::{traversal::element::Element, TreeVariant};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

type Item<V, E> = <E as Element>::Item<NodePtr<V>>;
type Stack<V, E> = Vec<Item<V, E>>;

pub struct DfsIterPtr<V, E, S = Stack<V, E>>
where
    E: Element,
    V: TreeVariant,
    S: SoM<Stack<V, E>>,
{
    stack: S,
    phantom: PhantomData<(V, E)>,
}

impl<V, E, S> From<(S, NodePtr<V>)> for DfsIterPtr<V, E, S>
where
    E: Element,
    V: TreeVariant,
    S: SoM<Stack<V, E>>,
{
    fn from((mut stack, root): (S, NodePtr<V>)) -> Self {
        stack.get_mut().clear();
        stack.get_mut().push(E::from_root(root));
        Self {
            stack,
            phantom: PhantomData,
        }
    }
}

impl<V, E> Default for DfsIterPtr<V, E, Stack<V, E>>
where
    E: Element,
    V: TreeVariant,
{
    fn default() -> Self {
        Self {
            stack: Vec::default(),
            phantom: PhantomData,
        }
    }
}

impl<V, E> Clone for DfsIterPtr<V, E, Stack<V, E>>
where
    E: Element,
    V: TreeVariant,
{
    fn clone(&self) -> Self {
        Self {
            stack: self.stack.clone(),
            phantom: self.phantom.clone(),
        }
    }
}

impl<V, E, S> Iterator for DfsIterPtr<V, E, S>
where
    E: Element,
    V: TreeVariant,
    S: SoM<Stack<V, E>>,
{
    type Item = Item<V, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.get_mut().pop().map(|element| {
            let node_ptr = E::node_value(&element);
            let parent = unsafe { &*node_ptr.ptr() };
            let children_ptr = parent.next().children_ptr().cloned();
            let children = E::children(&element, children_ptr).rev();
            self.stack.get_mut().extend(children);
            element
        })
    }
}
