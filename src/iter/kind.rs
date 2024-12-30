use crate::{helpers::N, tree_variant::RefsChildren, Node, TreeVariant};
use core::marker::PhantomData;
use orx_pinned_vec::PinnedVec;
use orx_selfref_col::{MemoryPolicy, NodePtr, SelfRefCol};

pub trait IterKind<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type StackElement: StackElement<V>;

    type ValueFromNode: ValueFromNode<'a, V, M, P>;

    type YieldElement;

    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a;

    fn element(
        col: &'a SelfRefCol<V, M, P>,
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement;
}

// data

pub struct NodeVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterKind<'a, V, M, P> for NodeVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V, M, P>,
{
    type StackElement = NodePtr<V>;

    type ValueFromNode = D;

    type YieldElement = <Self::ValueFromNode as ValueFromNode<'a, V, M, P>>::Value;

    #[inline(always)]
    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        node(parent.node_ptr()).next().children_ptr().rev().cloned()
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement {
        D::value(col, node(&stack_element))
    }
}

// depth & data

pub struct NodeDepthVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterKind<'a, V, M, P> for NodeDepthVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V, M, P>,
{
    type StackElement = (usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (
        usize,
        <Self::ValueFromNode as ValueFromNode<'a, V, M, P>>::Value,
    );

    #[inline(always)]
    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .rev()
            .map(move |ptr| (depth, ptr.clone()))
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement {
        (stack_element.0, D::value(col, node(&stack_element.1)))
    }
}

// depth & sibling & data

pub struct NodeDepthSiblingVal<D>(PhantomData<D>);

impl<'a, V, M, P, D> IterKind<'a, V, M, P> for NodeDepthSiblingVal<D>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
    D: ValueFromNode<'a, V, M, P>,
{
    type StackElement = (usize, usize, NodePtr<V>);

    type ValueFromNode = D;

    type YieldElement = (
        usize,
        usize,
        <Self::ValueFromNode as ValueFromNode<'a, V, M, P>>::Value,
    );

    #[inline(always)]
    fn children(parent: &Self::StackElement) -> impl Iterator<Item = Self::StackElement> + 'a {
        let depth = parent.0 + 1;
        node(parent.node_ptr())
            .next()
            .children_ptr()
            .rev()
            .enumerate()
            .map(move |(i, ptr)| (depth, i, ptr.clone()))
    }

    #[inline(always)]
    fn element(
        col: &'a SelfRefCol<V, M, P>,
        stack_element: &Self::StackElement,
    ) -> Self::YieldElement {
        (
            stack_element.0,
            stack_element.1,
            D::value(col, node(&stack_element.2)),
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

pub trait StackElement<V: TreeVariant> {
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self;

    fn node_ptr(&self) -> &NodePtr<V>;
}

impl<V: TreeVariant> StackElement<V> for NodePtr<V> {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        root_ptr
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        self
    }
}

impl<V: TreeVariant> StackElement<V> for (usize, NodePtr<V>) {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        (0, root_ptr)
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.1
    }
}

impl<V: TreeVariant> StackElement<V> for (usize, usize, NodePtr<V>) {
    #[inline(always)]
    fn from_root_ptr(root_ptr: NodePtr<V>) -> Self {
        (0, 0, root_ptr)
    }

    #[inline(always)]
    fn node_ptr(&self) -> &NodePtr<V> {
        &self.2
    }
}

pub trait ValueFromNode<'a, V, M, P>
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value;

    fn value(col: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value;
}

pub struct NodeFromNode;

impl<'a, V, M, P> ValueFromNode<'a, V, M, P> for NodeFromNode
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = Node<'a, V, M, P>;

    #[inline(always)]
    fn value(col: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        Node::new(col, NodePtr::new(node as *const N<V>))
    }
}

pub struct DataFromNode;

impl<'a, V, M, P> ValueFromNode<'a, V, M, P> for DataFromNode
where
    V: TreeVariant + 'a,
    M: MemoryPolicy<V> + 'a,
    P: PinnedVec<N<V>> + 'a,
{
    type Value = &'a V::Item;

    #[inline(always)]
    fn value(_: &'a SelfRefCol<V, M, P>, node: &'a N<V>) -> Self::Value {
        node.data().expect("active tree node cannot be closed")
    }
}
