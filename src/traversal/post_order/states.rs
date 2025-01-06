use crate::TreeVariant;
use alloc::vec::Vec;
use orx_selfref_col::NodePtr;

pub type State<V> = (NodePtr<V>, usize); // (pointer, child_idx)
pub type States<V> = Vec<State<V>>;

pub fn set<V: TreeVariant>(states: &mut States<V>, depth: usize, pointer: NodePtr<V>) {
    match states.get_mut(depth) {
        Some(x) => *x = (pointer, 0),
        None => {
            debug_assert!(states.len() == depth);
            states.push((pointer, 0));
        }
    }
}

pub fn increment_child_idx<V: TreeVariant>(states: &mut States<V>, depth: usize) {
    states[depth].1 += 1;
}
