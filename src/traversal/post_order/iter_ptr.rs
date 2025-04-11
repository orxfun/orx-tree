use super::{post_enumeration::PostOrderEnumeration, states::State};
use crate::{TreeVariant, traversal::enumeration::Enumeration, tree_variant::RefsChildren};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_self_or::SoM;
use orx_selfref_col::NodePtr;

pub type Item<V, E> = <E as Enumeration>::Item<NodePtr<V>>;

pub struct PostOrderIterPtr<V, E, S = Vec<State<V>>>
where
    E: PostOrderEnumeration,
    V: TreeVariant,
    S: SoM<Vec<State<V>>>,
{
    states: S,
    depth: usize,
    phantom: PhantomData<(V, E)>,
}

impl<V, E, S> From<(S, NodePtr<V>)> for PostOrderIterPtr<V, E, S>
where
    E: PostOrderEnumeration,
    V: TreeVariant,
    S: SoM<Vec<State<V>>>,
{
    fn from((mut states, root): (S, NodePtr<V>)) -> Self {
        states.get_mut().clear();
        states.get_mut().push((root, 0));
        Self {
            states,
            depth: 0,
            phantom: PhantomData,
        }
    }
}

impl<V, E> Default for PostOrderIterPtr<V, E, Vec<State<V>>>
where
    E: PostOrderEnumeration,
    V: TreeVariant,
{
    fn default() -> Self {
        Self {
            states: Vec::default(),
            depth: 0,
            phantom: PhantomData,
        }
    }
}

impl<V, E> Clone for PostOrderIterPtr<V, E, Vec<State<V>>>
where
    E: PostOrderEnumeration,
    V: TreeVariant,
{
    fn clone(&self) -> Self {
        Self {
            states: self.states.clone(),
            depth: self.depth,
            phantom: PhantomData,
        }
    }
}

// iterator

impl<V, E, S> PostOrderIterPtr<V, E, S>
where
    E: PostOrderEnumeration,
    V: TreeVariant,
    S: SoM<Vec<State<V>>>,
{
    fn current(&self) -> Option<&State<V>> {
        match self.depth < usize::MAX {
            true => self.states.get_ref().get(self.depth),
            false => None,
        }
    }

    fn move_deeper(&mut self, child: NodePtr<V>) {
        self.depth += 1;
        super::states::set(self.states.get_mut(), self.depth, child);
    }

    fn move_shallower(&mut self) {
        match self.depth {
            0 => self.depth = usize::MAX,
            _ => {
                self.depth -= 1;
                super::states::increment_child_idx(self.states.get_mut(), self.depth);
            }
        }
    }
}

impl<V, E, S> Iterator for PostOrderIterPtr<V, E, S>
where
    E: PostOrderEnumeration,
    V: TreeVariant,
    S: SoM<Vec<State<V>>>,
{
    type Item = Item<V, E>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current() {
                None => return None,
                Some((ptr, child_idx)) => {
                    let node = unsafe { &*ptr.ptr() };
                    let child_ptr = node.next().get_ptr(*child_idx).cloned();
                    match child_ptr {
                        Some(child_ptr) => self.move_deeper(child_ptr),
                        None => {
                            let output = Some(E::create_post_item(
                                ptr.clone(),
                                self.depth,
                                self.states.get_ref(),
                            ));
                            self.move_shallower();
                            return output;
                        }
                    }
                }
            }
        }
    }
}
