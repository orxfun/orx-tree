#![allow(clippy::needless_lifetimes)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
use marker::PhantomData;
use std::*;

trait Variant {
    type Item;
}

trait NodeRef<V: Variant> {
    fn as_cloned_subtree(&self) -> ClonedFromNode<V, Self>
    where
        V::Item: Clone,
        Self: Sized,
    {
        ClonedFromNode {
            node: self,
            p: PhantomData,
        }
    }
}

struct Tree<V: Variant> {
    p: PhantomData<V>,
}

struct NodeIdx<V: Variant> {
    p: PhantomData<V>,
}
impl<V: Variant> NodeIdx<V> {
    fn as_cloned_subtree(&self) -> ClonedFromIdx<V>
    where
        V::Item: Clone,
        Self: Sized,
    {
        let idx: NodeIdx<V> = Self { p: PhantomData };
        ClonedFromIdx { idx }
    }

    fn into_subtree(&self) -> MovedFromIdx<V> {
        MovedFromIdx {
            idx: Self { p: PhantomData },
        }
    }
}

struct Node<V: Variant> {
    p: PhantomData<V>,
}

#[derive(Clone, Copy, Debug, Default)]
struct NodeMut<V: Variant> {
    p: PhantomData<V>,
}
impl<V: Variant> NodeMut<V> {
    fn into_subtree(&mut self) -> MovedFromNode<V> {
        MovedFromNode { node: self }
    }
}

impl<V: Variant> NodeRef<V> for Node<V> {}
impl<V: Variant> NodeRef<V> for NodeMut<V> {}

impl<V: Variant> NodeMut<V> {
    fn push_child(&mut self, value: V::Item) {}

    fn push_sibling(&mut self, value: V::Item) {}

    fn push_children(&mut self, values: impl IntoIterator<Item = V::Item>) {}

    fn extend_siblings(&mut self, values: impl IntoIterator<Item = V::Item>) {}

    fn push_parent(&mut self, value: V::Item) {}

    fn append_child(&mut self, node: &impl SubTree<V::Item>) {}

    fn append_sibling(&mut self, node: &impl SubTree<V::Item>) {}
}

pub trait SubTree<T> {}

struct ClonedFromNode<'a, V, N>
where
    V: Variant,
    N: NodeRef<V>,
    V::Item: Clone,
{
    node: &'a N,
    p: PhantomData<V>,
}
impl<'a, V, N> SubTree<V::Item> for ClonedFromNode<'a, V, N>
where
    V: Variant,
    N: NodeRef<V>,
    V::Item: Clone,
{
}

struct ClonedFromIdx<V>
where
    V: Variant,
    V::Item: Clone,
{
    idx: NodeIdx<V>,
}
impl<V> SubTree<V::Item> for ClonedFromIdx<V>
where
    V: Variant,
    V::Item: Clone,
{
}

struct MovedFromNode<'a, V>
where
    V: Variant,
{
    node: &'a mut NodeMut<V>,
}
impl<'a, V> SubTree<V::Item> for MovedFromNode<'a, V> where V: Variant {}

struct MovedFromIdx<V>
where
    V: Variant,
{
    idx: NodeIdx<V>,
}
impl<V> SubTree<V::Item> for MovedFromIdx<V> where V: Variant {}

impl<V> SubTree<V::Item> for Tree<V> where V: Variant {}
