use crate::{
    memory::MemoryPolicy, pinned_storage::PinnedStorage, traversal::traverser_core::TraverserCore,
    Node, NodeMut, NodeRef, Traversal, Traverser, Tree, TreeVariant,
};
use core::fmt::Debug;

impl<V, M, P> Debug for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut depth_first_sequence = alloc::vec::Vec::new();
        let mut t = Traversal.dfs().over_nodes().with_depth();
        let root = self.get_root();
        let mut num_leaves = 0usize;
        let mut max_depth = 0usize;
        let mut total_num_children = 0usize;
        let mut total_depth = 0usize;

        if let Some(root) = root.as_ref() {
            for (depth, node) in t.iter(root) {
                depth_first_sequence.push((depth, node.data()));

                total_num_children += node.num_children();
                total_depth += depth;

                if depth > max_depth {
                    max_depth = depth;
                }

                if node.is_leaf() {
                    num_leaves += 1;
                }
            }
        }

        let avg_degree = (100 * total_num_children / self.len()) as f64 / 100.0;
        let avg_non_leaf_degree =
            (100 * total_num_children / (self.len() - num_leaves)) as f64 / 100.0;
        let avg_depth = (100 * total_depth / self.len()) as f64 / 100.0;

        f.debug_struct("Tree")
            .field("len", &self.len())
            .field("max_depth", &max_depth)
            .field("num_leaves", &num_leaves)
            .field("avg_depth", &avg_depth)
            .field("avg_degree", &avg_degree)
            .field("avg_non_leaf_degree", &avg_non_leaf_degree)
            .field(
                "depth_value_pairs_in_depth_first_sequence",
                &depth_first_sequence,
            )
            .finish()
    }
}

// nodes

impl<V, M, P> Debug for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt_node(f, self)
    }
}

impl<V, M, P> Debug for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        fmt_node(f, self)
    }
}

fn fmt_node<'a, V, M, P>(
    f: &mut core::fmt::Formatter<'_>,
    node: &impl NodeRef<'a, V, M, P>,
) -> core::fmt::Result
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Debug,
{
    f.debug_struct("Node")
        .field("is_root", &node.is_root())
        .field("is_leaf", &node.is_leaf())
        .field("sibling_idx", &node.sibling_idx())
        .field("num_children", &node.num_children())
        .field("depth", &node.depth())
        .field("height", &node.height())
        .field("data", node.data())
        .finish()
}
