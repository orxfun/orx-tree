use super::Stack;
use crate::{
    helpers::N,
    traversal::{Enumeration, Traverser, Val},
    NodeMut, NodeRef, TreeVariant,
};
use alloc::vec::Vec;
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

// pub struct Dfs<V: TreeVariant, E: Element = Val> {
//     stack: Stack<V, E>,
// }

// impl<V, E> Traverser<V> for Dfs<V, E>
// where
//     V: TreeVariant,
//     E: Element,
// {
//     type ItemKind = E;

//     type NodeItem<'a, M, P>
//         = usize
//     where
//         M: MemoryPolicy<V> + 'a,
//         P: PinnedVec<N<V>> + 'a,
//         V: 'a,
//         Self: 'a;

//     fn iter<'a, M, P>(
//         &mut self,
//         node: &impl NodeRef<'a, V, M, P>,
//     ) -> impl Iterator<Item = <Self::ItemKind as Element>::Item<Self::NodeItem<'a, M, P>>>
//     where
//         V: TreeVariant + 'a,
//         M: MemoryPolicy<V> + 'a,
//         P: PinnedVec<N<V>> + 'a,
//         Self: 'a,
//     {
//         todo!()
//     }
// }
