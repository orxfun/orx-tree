use crate::{
    Dfs, MemoryPolicy, Node, NodeMut, NodeRef, Traversal, Traverser, Tree, TreeVariant,
    pinned_storage::PinnedStorage, traversal::OverDepthSiblingIdxNode,
};
use alloc::string::ToString;
use alloc::vec::Vec;
use core::fmt::Display;

impl<V, M, P> Display for Node<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Display,
{
    /// Creates a convenient-to-read string representation of the tree or a subtree rooted at a node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //  ╱ ╲   ╱ ╲
    /// // 4   5 6   7
    /// // |     |  ╱ ╲
    /// // 8     9 10  11
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// let expected_str = r#"1
    /// ├──2
    /// │  ├──4
    /// │  │  └──8
    /// │  └──5
    /// └──3
    ///    ├──6
    ///    │  └──9
    ///    └──7
    ///       ├──10
    ///       └──11
    /// "#;
    /// assert_eq!(tree.to_string(), expected_str);
    ///
    /// let expected_str = r#"3
    /// ├──6
    /// │  └──9
    /// └──7
    ///    ├──10
    ///    └──11
    /// "#;
    /// println!("{}", tree.node(id3).to_string());
    /// assert_eq!(tree.node(id3).to_string(), expected_str);
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut t = Traversal.dfs().over_nodes().with_depth().with_sibling_idx();
        display_node(f, self, &mut t)
    }
}

impl<V, M, P> Display for NodeMut<'_, V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Display,
{
    /// Creates a convenient-to-read string representation of the tree or a subtree rooted at a node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //  ╱ ╲   ╱ ╲
    /// // 4   5 6   7
    /// // |     |  ╱ ╲
    /// // 8     9 10  11
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// let expected_str = r#"1
    /// ├──2
    /// │  ├──4
    /// │  │  └──8
    /// │  └──5
    /// └──3
    ///    ├──6
    ///    │  └──9
    ///    └──7
    ///       ├──10
    ///       └──11
    /// "#;
    /// assert_eq!(tree.to_string(), expected_str);
    ///
    /// let expected_str = r#"3
    /// ├──6
    /// │  └──9
    /// └──7
    ///    ├──10
    ///    └──11
    /// "#;
    /// println!("{}", tree.node(id3).to_string());
    /// assert_eq!(tree.node(id3).to_string(), expected_str);
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut t = Traversal.dfs().over_nodes().with_depth().with_sibling_idx();
        display_node(f, self, &mut t)
    }
}

impl<V, M, P> Display for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Display,
{
    /// Creates a convenient-to-read string representation of the tree or a subtree rooted at a node.
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// //      1
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   2     3
    /// //  ╱ ╲   ╱ ╲
    /// // 4   5 6   7
    /// // |     |  ╱ ╲
    /// // 8     9 10  11
    ///
    /// let mut tree = DynTree::new(1);
    ///
    /// let mut root = tree.root_mut();
    /// let [id2, id3] = root.push_children([2, 3]);
    /// let [id4, _] = tree.node_mut(id2).push_children([4, 5]);
    /// tree.node_mut(id4).push_child(8);
    /// let [id6, id7] = tree.node_mut(id3).push_children([6, 7]);
    /// tree.node_mut(id6).push_child(9);
    /// tree.node_mut(id7).push_children([10, 11]);
    ///
    /// let expected_str = r#"1
    /// ├──2
    /// │  ├──4
    /// │  │  └──8
    /// │  └──5
    /// └──3
    ///    ├──6
    ///    │  └──9
    ///    └──7
    ///       ├──10
    ///       └──11
    /// "#;
    /// assert_eq!(tree.to_string(), expected_str);
    ///
    /// let expected_str = r#"3
    /// ├──6
    /// │  └──9
    /// └──7
    ///    ├──10
    ///    └──11
    /// "#;
    /// println!("{}", tree.node(id3).to_string());
    /// assert_eq!(tree.node(id3).to_string(), expected_str);
    /// ```
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.get_root() {
            Some(root) => {
                let mut t = Traversal.dfs().over_nodes().with_depth().with_sibling_idx();
                display_node(f, &root, &mut t)
            }
            None => Ok(()),
        }
    }
}

fn display_node<'a, V, M, P>(
    f: &mut core::fmt::Formatter<'_>,
    node: &'a impl NodeRef<'a, V, M, P>,
    t: &'a mut Dfs<OverDepthSiblingIdxNode>,
) -> core::fmt::Result
where
    V: TreeVariant + 'a,
    M: MemoryPolicy,
    P: PinnedStorage,
    V::Item: Display,
{
    fn set_depth(depths: &mut Vec<bool>, depth: usize, continues: bool) {
        match depth < depths.len() {
            true => depths[depth] = continues,
            false => depths.push(continues),
        }
    }

    let mut depths: Vec<bool> = Default::default();
    let mut iter = node.walk_with(t);

    if let Some((_, _, root)) = iter.next() {
        writeln!(f, "{}", root.data().to_string().as_str())?;
        set_depth(&mut depths, 0, true);

        for (depth, sibling_idx, node) in iter {
            let is_last_sibling = node.num_siblings() == sibling_idx + 1;

            set_depth(&mut depths, depth, true);
            set_depth(&mut depths, depth - 1, !is_last_sibling);

            for d in depths.iter().take(depth - 1) {
                match d {
                    true => write!(f, "│")?,
                    false => write!(f, " ")?,
                }
                write!(f, "  ")?;
            }

            let first_depth_char = match is_last_sibling {
                true => '└',
                false => '├',
            };

            write!(f, "{first_depth_char}")?;
            write!(f, "──")?;

            writeln!(f, "{}", node.data().to_string().as_str())?;
        }
    }

    Ok(())
}
