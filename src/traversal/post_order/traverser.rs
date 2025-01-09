use super::{
    into_iter::PostOrderIterInto, iter_mut::PostOrderIterMut, iter_ptr::PostOrderIterPtr,
    iter_ref::PostOrderIterRef, states::States,
};
use crate::{
    memory::MemoryPolicy,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemMut, OverMut},
        Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};
use core::marker::PhantomData;

/// A post order traverser ([Wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Post-order,_LRN)).
///
/// A traverser can be created once and used to traverse over trees multiple times without
/// requiring additional memory allocation.
///
/// # Construction
///
/// A post order traverser can be created,
/// * either by using Default trait and providing its two generic type parameters
///   * `PostOrder::<_, OverData>::default()` or `PostOrder::<_, OverDepthSiblingIdxData>::default()`, or
///   * `PostOrder::<Dyn<u64>, OverData>::default()` or `PostOrder::<Dary<2, String>, OverDepthSiblingIdxData>::default()`
///     if we want the complete type signature.
/// * or by using the [`Traversal`] type.
///   * `Traversal.post_order()` or `Traversal.post_order().with_depth().with_sibling_idx()`.
///
/// [`Traversal`]: crate::Traversal
pub struct PostOrder<O = OverData>
where
    O: Over,
{
    states: States,
    phantom: PhantomData<O>,
}

impl<O> Default for PostOrder<O>
where
    O: Over,
{
    fn default() -> Self {
        Self {
            states: Default::default(),
            phantom: PhantomData,
        }
    }
}

impl<O> Traverser<O> for PostOrder<O>
where
    O: Over,
{
    type IntoOver<O2>
        = PostOrder<O2>
    where
        O2: Over;

    fn iter<'a, V, M, P>(
        &'a mut self,
        node: &'a impl NodeRef<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItem<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: 'a,
        Self: 'a,
    {
        let root = node.node_ptr().clone();
        let states = self.states.for_variant::<V>();
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((states, root));
        PostOrderIterRef::from((node.col(), iter_ptr))
    }

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        PostOrder {
            states: self.states,
            phantom: PhantomData,
        }
    }

    fn iter_mut<'a, V, M, P>(
        &'a mut self,
        node_mut: &'a mut NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemMut<'a, V, O, M, P>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut + 'a,
        Self: 'a,
    {
        let root = node_mut.node_ptr().clone();
        let states = self.states.for_variant::<V>();
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((states, root));
        unsafe { PostOrderIterMut::from((node_mut.col(), iter_ptr)) }
    }

    fn into_iter<'a, V, M, P>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = crate::traversal::over_mut::OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut + 'a,
        Self: 'a,
    {
        let (col, root) = node_mut.into_inner();
        let states = self.states.for_variant::<V>();
        let iter_ptr = PostOrderIterPtr::<V, O::Enumeration, _>::from((states, root.clone()));
        unsafe { PostOrderIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}

#[test]
fn ghi() {
    use crate::*;
    use alloc::vec::Vec;

    //      1
    //     ╱ ╲
    //    ╱   ╲
    //   2     3
    //  ╱ ╲   ╱ ╲
    // 4   5 6   7
    // |     |  ╱ ╲
    // 8     9 10  11

    let mut tr = PostOrder::<OverData>::default();

    let mut tree1 = DynTree::<i32>::new(1);

    let mut root = tree1.root_mut().unwrap();
    let [id2, tree1_id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree1);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree1).push(8);

    let mut n3 = tree1_id3.node_mut(&mut tree1);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree1).push(9);
    id7.node_mut(&mut tree1).extend([10, 11]);

    // second tree

    let mut tree2 = DynTree::<u64>::new(1);

    let mut root = tree2.root_mut().unwrap();
    let [id2, tree2_id3] = root.grow([2, 3]);

    let mut n2 = id2.node_mut(&mut tree2);
    let [id4, _] = n2.grow([4, 5]);

    id4.node_mut(&mut tree2).push(8);

    let mut n3 = tree2_id3.node_mut(&mut tree2);
    let [id6, id7] = n3.grow([6, 7]);

    id6.node_mut(&mut tree2).push(9);
    id7.node_mut(&mut tree2).extend([10, 11]);

    let mut root2 = tree2.root_mut().unwrap();
    for x in tr.iter_mut(&mut root2) {
        *x += 100;
    }

    // data

    let root = tree1.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).copied().collect();
    assert_eq!(vals, [8, 4, 5, 2, 9, 6, 10, 11, 7, 3, 1]);

    let root = tree2.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).copied().collect();
    assert_eq!(
        vals,
        [108, 104, 105, 102, 109, 106, 110, 111, 107, 103, 101]
    );

    // data into

    let tree1_n3 = tree1_id3.node_mut(&mut tree1);
    let vals: Vec<_> = tr.into_iter(tree1_n3).collect();
    assert_eq!(vals, [9, 6, 10, 11, 7, 3]);
    let tree1_rem: Vec<_> = tr.iter(&tree1.root().unwrap()).copied().collect();
    assert_eq!(tree1_rem, [8, 4, 5, 2, 1]);

    let tree2_n3 = tree2_id3.node_mut(&mut tree2);
    let vals: Vec<_> = tr.into_iter(tree2_n3).collect();
    assert_eq!(vals, [109, 106, 110, 111, 107, 103]);

    let tree2_rem: Vec<_> = tr.iter(&tree2.root().unwrap()).copied().collect();
    assert_eq!(tree2_rem, [108, 104, 105, 102, 101]);

    // depth - data

    let mut tr = tr.with_depth();

    let root = tree1.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).map(|x| *x.1).collect();
    assert_eq!(vals, [8, 4, 5, 2, 1]);

    let root = tree2.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).map(|x| *x.1).collect();
    assert_eq!(vals, [108, 104, 105, 102, 101]);
}
