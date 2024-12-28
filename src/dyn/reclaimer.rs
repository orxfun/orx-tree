use super::Dyn;
use crate::helpers::N;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{CoreCol, MemoryReclaimer, NodePtr};

#[derive(Clone, Default)]
pub struct DynReclaimer;

impl<T> MemoryReclaimer<Dyn<T>> for DynReclaimer {
    fn reclaim_nodes<P>(col: &mut CoreCol<Dyn<T>, P>) -> bool
    where
        P: PinnedVec<N<Dyn<T>>>,
    {
        let mut any_swapped = false;

        // SAFETY: lifetimes of `forward` and `backward` iterators are limited to this method
        // which is shorter than the lifetime of the `col`
        let forward = unsafe { col.nodes().iter_ptr() };
        let mut backward = unsafe { col.nodes().iter_ptr_rev() };
        let mut o = col.nodes().len();

        for (v, vacant_ptr) in forward.enumerate() {
            if v >= o {
                break;
            }

            if unsafe { &*vacant_ptr }.is_closed() {
                while o > v {
                    o -= 1;
                    let occupied_ptr = backward.next().expect("cannot be consumed before forward");

                    if unsafe { &*occupied_ptr }.is_active() {
                        any_swapped = true;
                        swap(col, vacant_ptr, occupied_ptr);
                        break;
                    }
                }
            }
        }

        any_swapped
    }
}

fn swap<P, T>(col: &mut CoreCol<Dyn<T>, P>, vacant: *const N<Dyn<T>>, occupied: *const N<Dyn<T>>)
where
    P: PinnedVec<N<Dyn<T>>>,
{
    // occupied.parent.child => vacant
    if let Some(parent) = (unsafe { &*occupied }).prev().get() {
        let parent = col.node_mut(parent);
        let children = parent.next_mut();

        let child = children
            .iter_mut()
            .find(|x| x.ptr() == occupied)
            .expect("valid tree state ensures this");

        *child = NodePtr::new(vacant);
    }

    // occupied.children.parent => vacant
    for child in (unsafe { &*occupied }).next().iter() {
        let child_mut = unsafe { child.node_mut() };
        let parent = child_mut.prev_mut();
        parent.set_some(NodePtr::new(vacant));
    }

    // root => vacant

    if occupied == col.ends().get().expect("nonempty tree").ptr() {
        col.ends_mut().set_some(NodePtr::new(vacant));
    }

    core::mem::swap(unsafe { &mut *(vacant as *mut N<Dyn<T>>) }, unsafe {
        &mut *(occupied as *mut N<Dyn<T>>)
    });
}
