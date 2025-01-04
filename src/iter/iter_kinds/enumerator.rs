use crate::TreeVariant;
use orx_selfref_col::NodePtr;

pub trait Enumerator {
    type Output<Data>;

    fn from_ptr<V, M, Data>(output_ptr: Self::Output<NodePtr<V>>, map: M) -> Self::Output<Data>
    where
        V: TreeVariant,
        M: FnOnce(NodePtr<V>) -> Data;
}

pub struct EnumeratorNone;
impl Enumerator for EnumeratorNone {
    type Output<Data> = Data;

    fn from_ptr<V, M, Data>(output_ptr: Self::Output<NodePtr<V>>, map: M) -> Self::Output<Data>
    where
        V: TreeVariant,
        M: FnOnce(NodePtr<V>) -> Data,
    {
        map(output_ptr)
    }
}

pub struct EnumeratorDepth;
impl Enumerator for EnumeratorDepth {
    type Output<Data> = (usize, Data);

    fn from_ptr<V, M, Data>(output_ptr: Self::Output<NodePtr<V>>, map: M) -> Self::Output<Data>
    where
        V: TreeVariant,
        M: FnOnce(NodePtr<V>) -> Data,
    {
        (output_ptr.0, map(output_ptr.1))
    }
}

pub struct EnumeratorSibling;
impl Enumerator for EnumeratorSibling {
    type Output<Data> = (usize, Data);

    fn from_ptr<V, M, Data>(output_ptr: Self::Output<NodePtr<V>>, map: M) -> Self::Output<Data>
    where
        V: TreeVariant,
        M: FnOnce(NodePtr<V>) -> Data,
    {
        (output_ptr.0, map(output_ptr.1))
    }
}

pub struct EnumeratorDepthSibling;
impl Enumerator for EnumeratorDepthSibling {
    type Output<Data> = (usize, usize, Data);

    fn from_ptr<V, M, Data>(output_ptr: Self::Output<NodePtr<V>>, map: M) -> Self::Output<Data>
    where
        V: TreeVariant,
        M: FnOnce(NodePtr<V>) -> Data,
    {
        (output_ptr.0, output_ptr.1, map(output_ptr.2))
    }
}
