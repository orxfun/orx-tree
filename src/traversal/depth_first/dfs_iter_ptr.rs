use crate::{traversal::element::Element, TreeVariant};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

type Item<V, E> = <E as Element>::Item<NodePtr<V>>;
type Stack<V, E> = Vec<Item<V, E>>;

pub struct DfsIterPtr<S, V, E>
where
    E: Element,
    V: TreeVariant,
    S: SoM<Stack<V, E>>,
{
    stack: S,
    phantom: PhantomData<(V, E)>,
}

impl<S, V, E> Iterator for DfsIterPtr<S, V, E>
where
    E: Element,
    V: TreeVariant,
    S: SoM<Stack<V, E>>,
{
    type Item = Item<V, E>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stack.get_mut().pop().map(|parent| {
            // let parent = unsafe{&*parent.0.}
            // node(parent.node_ptr()).next().children_ptr().rev().cloned()
            // unsafe { &*node_ptr.ptr() }

            // let children = K::children_rev(&parent);
            // self.stack.get_mut().extend(children);
            // K::element(self.col, &parent)
            todo!()
        })
    }
}
