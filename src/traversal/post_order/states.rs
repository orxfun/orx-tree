use crate::{Dyn, TreeVariant};
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

pub type State<V> = (NodePtr<V>, usize); // (pointer, child_idx)

pub fn set<V: TreeVariant>(states: &mut Vec<State<V>>, depth: usize, pointer: NodePtr<V>) {
    match states.get_mut(depth) {
        Some(x) => *x = (pointer, 0),
        None => {
            debug_assert!(states.len() == depth);
            states.push((pointer, 0));
        }
    }
}

pub fn increment_child_idx<V: TreeVariant>(states: &mut [State<V>], depth: usize) {
    states[depth].1 += 1;
}

#[derive(Default)]
pub struct States {
    states: Vec<State<Dyn<i32>>>,
}

impl States {
    pub(crate) fn for_variant<V>(&mut self) -> &mut Vec<State<V>>
    where
        V: TreeVariant,
    {
        // # SAFETY: Size and layout of stored elements in the states
        // do not change => (NodePtr<V>, usize)
        //
        // Since NodePtr<V> under the hood contains only one raw pointer,
        // memory size and layout of elements are independent of V.
        unsafe { core::mem::transmute(&mut self.states) }
    }
}
