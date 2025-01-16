pub trait SubTree<T> {
    fn dfs_iter(&mut self) -> impl IntoIterator<Item = (usize, T)>;
}
