#![allow(dead_code, unused_mut, unused_imports, unused_variables)]

use crate::Dary;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{CoreCol, MemoryReclaimer, Node, NodePtr};

#[derive(Clone, Default)]
pub struct DaryReclaimer;

impl<const D: usize, T> MemoryReclaimer<Dary<D, T>> for DaryReclaimer {
    fn reclaim_nodes<P>(col: &mut CoreCol<Dary<D, T>, P>) -> bool
    where
        P: PinnedVec<Node<Dary<D, T>>>,
    {
        let mut nodes_moved = false;

        if let Some(mut occupied_ptr) = col.ends().get() {
            // let mut prev = core::ptr::null();

            // SAFETY: lifetime of `forward` iterator is limited to this method
            // which is shorter than the lifetime of the `col`
            let forward = unsafe { col.nodes().iter_ptr() }.enumerate();

            for (v, vacant_ptr) in forward {
                if unsafe { &*vacant_ptr }.is_closed() {
                    loop {
                        let o = col.position_of_unchecked(&occupied_ptr);

                        let children = col.node(&occupied_ptr).next();

                        for next in children.iter_somes() {
                            //
                        }

                        // let next = col.node(&occupied_ptr).next().get();

                        // let swapped = o > v;
                        // match swapped {
                        //     true => {
                        //         nodes_moved = true;
                        //         swap(col, vacant_ptr, occupied_ptr.ptr(), prev);
                        //         prev = vacant_ptr;
                        //     }
                        //     false => prev = occupied_ptr.ptr(),
                        // }

                        // match next {
                        //     Some(next) => occupied_ptr = next,
                        //     None => return nodes_moved,
                        // }

                        // if swapped {
                        //     break;
                        // }
                    }
                }
            }
        }

        nodes_moved
    }
}
