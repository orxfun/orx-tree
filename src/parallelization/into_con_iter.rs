use crate::{aliases::N, TreeVariant};

pub struct TreeIntoConIter<V, I>
where
    V: TreeVariant,
    I: Iterator<Item = N<V>>,
{
    iter: I,
}
