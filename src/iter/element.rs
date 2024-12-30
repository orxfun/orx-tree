use crate::{helpers::N, tree_variant::RefsChildren, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr};

pub trait IterElement<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type StackElement: HasPtr<V>;

    type ValueFromNode: ValueFromNode<'a, V>;

    type YieldElement;

    fn children(parent: &'a Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a;

    fn element(stack_element: &Self::StackElement) -> Self::YieldElement;
}

// data

pub struct NodeVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterElement<'a, V, M, P> for NodeVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V>,
{
    type StackElement = NodePtr<V>;

    type ValueFromNode = D;

    type YieldElement = <Self::ValueFromNode as ValueFromNode<'a, V>>::Value;

    fn children(parent: &'a Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        node(parent.node_ptr()).next().children_ptr().cloned()
    }

    fn element(stack_element: &Self::StackElement) -> Self::YieldElement {
        D::value(node(&stack_element))
    }
}

// depth & data

pub struct NodeDepthVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterElement<'a, V, M, P> for NodeDepthVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V>,
{
    type StackElement = (usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (usize, <Self::ValueFromNode as ValueFromNode<'a, V>>::Value);

    fn children(parent: &'a Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .map(move |ptr| (depth, ptr.clone()))
    }

    fn element(stack_element: &Self::StackElement) -> Self::YieldElement {
        (stack_element.0, D::value(node(&stack_element.1)))
    }
}

// depth & sibling & data

pub struct NodeDepthSiblingVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterElement<'a, V, M, P> for NodeDepthSiblingVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V>,
{
    type StackElement = (usize, usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (
        usize,
        usize,
        <Self::ValueFromNode as ValueFromNode<'a, V>>::Value,
    );

    fn children(parent: &'a Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .enumerate()
            .map(move |(i, ptr)| (depth, i, ptr.clone()))
    }

    fn element(stack_element: &Self::StackElement) -> Self::YieldElement {
        (
            stack_element.0,
            stack_element.1,
            D::value(node(&stack_element.2)),
        )
    }
}

// helpers

/// # Safety
///
/// It is safe to use this method within this file which guarantees that:
/// * node_ptr points to an active node with a valid pointer.
/// * created node reference is not leaked by the next method of the iterator.
///
/// node_ptr points to a node belonging to the collection `col`.
/// The collection is referenced for the lifetime of 'a; therefore, the
/// node that the pointer is pointing at will be valid for at least 'a.
#[inline(always)]
fn node<'a, V: TreeVariant>(node_ptr: &NodePtr<V>) -> &'a N<V> {
    unsafe { &*node_ptr.ptr() }
}

pub trait HasPtr<V: TreeVariant> {
    fn node_ptr(&self) -> &NodePtr<V>;
}

impl<V: TreeVariant> HasPtr<V> for NodePtr<V> {
    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        self
    }
}

impl<T, V: TreeVariant> HasPtr<V> for (T, NodePtr<V>) {
    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.1
    }
}

impl<T, U, V: TreeVariant> HasPtr<V> for (T, U, NodePtr<V>) {
    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.2
    }
}

pub trait ValueFromNode<'a, V>
where
    V: TreeVariant,
{
    type Value;

    fn value(node: &'a N<V>) -> Self::Value;
}

pub struct NodeFromNode;

impl<'a, V: TreeVariant + 'a> ValueFromNode<'a, V> for NodeFromNode {
    type Value = &'a N<V>;

    #[inline(always)]
    fn value(node: &'a N<V>) -> Self::Value {
        node
    }
}

pub struct DataFromNode;

impl<'a, V: TreeVariant + 'a> ValueFromNode<'a, V> for DataFromNode {
    type Value = &'a V::Item;

    #[inline(always)]
    fn value(node: &'a N<V>) -> Self::Value {
        node.data().expect("active tree node cannot be closed")
    }
}
