use crate::{
    pinned_storage::PinnedStorage, MemoryPolicy, NodeRef, Traversal, Traverser, Tree, TreeVariant,
};
use core::marker::PhantomData;
use serde::{de::Visitor, ser::SerializeSeq, Deserialize, Serialize};

// Serialize

impl<V, M, P> Serialize for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
    V::Item: Serialize,
{
    /// Serializes the tree into linearized [`DepthFirstSequence`].
    ///
    /// [`DepthFirstSequence`]: crate::DepthFirstSequence
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let tree = BinaryTree::empty();
    /// let json = serde_json::to_string(&tree).unwrap();
    /// assert_eq!(json, "[]");
    ///
    /// let tree = DynTree::new(10);
    /// let json = serde_json::to_string(&tree).unwrap();
    /// assert_eq!(json, "[[0,10]]");
    ///
    /// //      0
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     2
    /// //  ╱     ╱ ╲
    /// // 3     4   5
    /// // |         |
    /// // 6         7
    /// let mut tree = DaryTree::<4, _>::new(0);
    /// let [id1, id2] = tree.root_mut().push_children([1, 2]);
    /// let id3 = tree.node_mut(&id1).push_child(3);
    /// tree.node_mut(&id3).push_child(6);
    /// let [_, id5] = tree.node_mut(&id2).push_children([4, 5]);
    /// tree.node_mut(&id5).push_child(7);
    ///
    /// let json = serde_json::to_string(&tree).unwrap();
    /// assert_eq!(json, "[[0,0],[1,1],[2,3],[3,6],[1,2],[2,4],[2,5],[3,7]]");
    /// ```
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;
        if let Some(root) = self.get_root() {
            let mut t = Traversal.dfs().with_depth();
            for x in root.walk_with(&mut t) {
                seq.serialize_element(&x)?;
            }
        }
        seq.end()
    }
}

// Deserialize

/// Tree deserializer using the linear DepthFirstSequence representation of a tree.
struct TreeDeserializer<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
{
    phantom: PhantomData<(V, M, P)>,
}

impl<'de, V, M, P> Visitor<'de> for TreeDeserializer<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
    V::Item: Deserialize<'de>,
{
    type Value = Tree<V, M, P>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {}

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let err = |x| Err(serde::de::Error::custom(x));
        use alloc::format;

        let mut tree = Tree::empty();

        if let Some(x) = seq.next_element()? {
            let (depth, value): (usize, V::Item) = x;
            if depth != 0 {
                return err(format!("First element of DepthFirstSequence (root of the tree) must have depth 0; however, received a depth of {}. Following is an example valid sequence of (depth, value) pairs: [[0, 0], [1, 1], [2, 3], [3, 6], [1, 2], [2, 4], [2, 5], [3, 7]].", depth));
            }
            tree.push_root(value);

            let mut current_depth = depth;

            let mut dst = tree.root_mut();

            while let Some((depth, value)) = seq.next_element::<(usize, V::Item)>()? {
                match depth > current_depth {
                    true => {
                        if depth > current_depth + 1 {
                            return err(format!("Let d1 and d2 be two consecutive depths in the depth-first sequence. Then, (i) d2=d1+1, (ii) d2=d1 or (iii) d2<d1 are valid cases. However, received the invalid case where d2>d1+1 (d1={}, d2={}). Please see DepthFirstSequenceError documentation for details. Following is an example valid sequence of (depth, value) pairs: [[0, 0], [1, 1], [2, 3], [3, 6], [1, 2], [2, 4], [2, 5], [3, 7]].", current_depth, depth));
                        }
                    }
                    false => {
                        let num_parent_moves = current_depth - depth + 1;
                        for _ in 0..num_parent_moves {
                            dst = dst.into_parent_mut().expect("in bounds");
                        }
                    }
                }
                let position = dst.num_children();
                dst.push_child(value);
                dst = dst.into_child_mut(position).expect("child exists");
                current_depth = depth;
            }
        }

        Ok(tree)
    }
}

impl<'de, V, M, P> Deserialize<'de> for Tree<V, M, P>
where
    V: TreeVariant,
    M: MemoryPolicy,
    P: PinnedStorage,
    P::PinnedVec<V>: Default,
    V::Item: Deserialize<'de>,
{
    /// Deserializes the tree from linearized [`DepthFirstSequence`].
    ///
    /// [`DepthFirstSequence`]: crate::DepthFirstSequence
    ///
    /// # Examples
    ///
    /// ```
    /// use orx_tree::*;
    ///
    /// let json = "[]";
    /// let result: Result<DaryTree<4, i32>, _> = serde_json::from_str(json);
    /// let tree = result.unwrap();
    /// assert!(tree.is_empty());
    ///
    /// let json = "[[0,10]]";
    /// let result: Result<DynTree<i32>, _> = serde_json::from_str(json);
    /// let tree = result.unwrap();
    /// assert_eq!(tree.len(), 1);
    /// assert_eq!(tree.root().data(), &10);
    ///
    /// //      0
    /// //     ╱ ╲
    /// //    ╱   ╲
    /// //   1     2
    /// //  ╱     ╱ ╲
    /// // 3     4   5
    /// // |         |
    /// // 6         7
    /// let json = "[[0, 0], [1, 1], [2, 3], [3, 6], [1, 2], [2, 4], [2, 5], [3, 7]]";
    /// let result: Result<BinaryTree<i32>, _> = serde_json::from_str(json);
    /// let tree = result.unwrap();
    /// let bfs_values: Vec<_> = tree.root().walk::<Bfs>().copied().collect();
    /// assert_eq!(bfs_values, [0, 1, 2, 3, 4, 5, 6, 7]);
    ///
    /// // errors
    ///
    /// // A. First element of DepthFirstSequence (root of the tree) must have depth 0;
    /// // however, received a depth of 1.
    /// let json = "[[1,10]]";
    /// let result: Result<DynTree<i32>, _> = serde_json::from_str(json);
    /// assert!(result.is_err());
    ///
    /// // B. Let d1 and d2 be two consecutive depths in the depth-first sequence.
    /// // Then, (i) d2=d1+1, (ii) d2=d1 or (iii) d2<d1 are valid cases.
    /// // However, received the invalid case where d2>d1+1 (d1=1, d2=3).
    /// let json = "[[0, 0], [1, 1], [3, 6], [1, 2], [2, 4], [2, 5], [3, 7]]";
    /// let result: Result<BinaryTree<i32>, _> = serde_json::from_str(json);
    /// assert!(result.is_err());
    /// ```
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(TreeDeserializer {
            phantom: PhantomData,
        })
    }
}
