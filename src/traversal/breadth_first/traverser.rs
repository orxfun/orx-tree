use super::{
    into_iter::BfsIterInto, iter_mut::BfsIterMut, iter_ptr::BfsIterPtr, iter_ref::BfsIterRef,
    queue::Queue,
};
use crate::{
    memory::MemoryPolicy,
    node_ref::NodeRefCore,
    pinned_storage::PinnedStorage,
    traversal::{
        over::{Over, OverData, OverItem},
        over_mut::{OverItemInto, OverItemMut, OverMut},
        traverser::Traverser,
    },
    NodeMut, NodeRef, TreeVariant,
};

/// A breadth first search traverser, also known as level-order
/// ([wikipedia](https://en.wikipedia.org/wiki/Tree_traversal#Breadth-first_search)).
///
/// A traverser can be created once and used to traverse over trees multiple times without
/// requiring additional memory allocation.
///
/// # Construction
///
/// A depth first traverser can be created,
/// * either by using Default trait and providing its two generic type parameters
///   * `Dfs::<_, OverData>::default()` or `Dfs::<_, OverDepthSiblingIdxData>::default()`, or
///   * `Dfs::<Dyn<u64>, OverData>::default()` or `Dfs::<Dary<2, String>, OverDepthSiblingIdxData>::default()`
///     if we want the complete type signature.
/// * or by using the [`Traversal`] type.
///   * `Traversal.dfs()` or `Traversal.dfs().with_depth().with_sibling_idx()`.
///
/// [`Traversal`]: crate::Traversal
pub struct Bfs<O = OverData>
where
    O: Over,
{
    queue: Queue<O::Enumeration>,
}

impl<O> Default for Bfs<O>
where
    O: Over,
{
    fn default() -> Self {
        Self {
            queue: Default::default(),
        }
    }
}

impl<O> Traverser<O> for Bfs<O>
where
    O: Over,
{
    type IntoOver<O2>
        = Bfs<O2>
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
        let queue = self.queue.for_variant::<V>();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((queue, root));
        BfsIterRef::from((node.col(), iter_ptr))
    }

    fn transform_into<O2: Over>(self) -> Self::IntoOver<O2> {
        Bfs::<O2>::default()
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
        let queue = self.queue.for_variant::<V>();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((queue, root));
        unsafe { BfsIterMut::from((node_mut.col(), iter_ptr)) }
    }

    fn into_iter<'a, V, M, P>(
        &'a mut self,
        node_mut: NodeMut<'a, V, M, P>,
    ) -> impl Iterator<Item = OverItemInto<'a, V, O>>
    where
        V: TreeVariant + 'a,
        M: MemoryPolicy,
        P: PinnedStorage,
        O: OverMut + 'a,
        Self: 'a,
    {
        let (col, root) = node_mut.into_inner();
        let queue = self.queue.for_variant::<V>();
        let iter_ptr = BfsIterPtr::<V, O::Enumeration, _>::from((queue, root.clone()));
        unsafe { BfsIterInto::<V, M, P, _, _>::from((col, iter_ptr, root)) }
    }
}

#[test]
fn abc() {
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

    let mut tr = Bfs::<OverData>::default();

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

    let mut tree2 = DaryTree::<4, i32>::new(1);

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
    assert_eq!(vals, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11]);

    let root = tree2.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).copied().collect();
    assert_eq!(
        vals,
        [101, 102, 103, 104, 105, 106, 107, 108, 109, 110, 111]
    );

    // data into

    let tree1_n3 = tree1_id3.node_mut(&mut tree1);
    let vals: Vec<_> = tr.into_iter(tree1_n3).collect();
    assert_eq!(vals, [3, 6, 7, 9, 10, 11]);
    let tree1_rem: Vec<_> = tr.iter(&tree1.root().unwrap()).copied().collect();
    assert_eq!(tree1_rem, [1, 2, 4, 5, 8]);

    let tree2_n3 = tree2_id3.node_mut(&mut tree2);
    let vals: Vec<_> = tr.into_iter(tree2_n3).collect();
    assert_eq!(vals, [103, 106, 107, 109, 110, 111]);
    let tree2_rem: Vec<_> = tr.iter(&tree2.root().unwrap()).copied().collect();
    assert_eq!(tree2_rem, [101, 102, 104, 105, 108]);

    // depth - data

    let mut tr = tr.with_depth();

    let root = tree1.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).map(|x| *x.1).collect();
    assert_eq!(vals, [1, 2, 4, 5, 8]);

    let root = tree2.root().unwrap();
    let vals: Vec<_> = tr.iter(&root).map(|x| *x.1).collect();
    assert_eq!(vals, [101, 102, 104, 105, 108]);
}
