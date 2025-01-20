pub trait EasyIter<T> {
    fn filter_to_vec<F>(self, func: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool;

    fn map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: FnMut(T) -> T2,
        B: FromIterator<T2>;

    fn filter_map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: FnMut(T) -> Option<T2>,
        B: FromIterator<T2>;
}

impl<I: IntoIterator<Item = T>, T> EasyIter<T> for I {
    fn filter_to_vec<F>(self, func: F) -> Vec<T>
    where
        F: FnMut(&T) -> bool,
    {
        self.into_iter().filter(func).collect()
    }

    fn filter_map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: FnMut(T) -> Option<T2>,
        B: FromIterator<T2>,
    {
        self.into_iter().filter_map(func).collect()
    }

    fn map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: FnMut(T) -> T2,
        B: FromIterator<T2>,
    {
        self.into_iter().map(func).collect()
    }
}
