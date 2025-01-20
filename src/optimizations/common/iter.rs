// use itertools::Itertools;

// TODO: potentially remove if not needed
// pub fn _apply_result<T, F>(nodes: Vec<T>, func: F) -> Result<Vec<T>>
// where
//     F: Fn(T) -> Result<Option<T>>,
// {
//     nodes
//         .into_iter()
//         .map(func)
//         .process_results(|iter| iter.flatten().collect())
// }

pub trait EasyIter<T> {
    fn filter_to_vec<F>(self, func: F) -> Vec<T>
    where
        F: Fn(&T) -> bool;

    fn map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> T2,
        B: FromIterator<T2>;

    fn filter_map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> Option<T2>,
        B: FromIterator<T2>;
}

impl<I: IntoIterator<Item = T>, T> EasyIter<T> for I {
    fn filter_to_vec<F>(self, func: F) -> Vec<T>
    where
        F: Fn(&T) -> bool,
    {
        self.into_iter().filter(func).collect()
    }

    fn filter_map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> Option<T2>,
        B: FromIterator<T2>,
    {
        self.into_iter().filter_map(func).collect()
    }

    fn map_to_vec<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> T2,
        B: FromIterator<T2>,
    {
        self.into_iter().map(func).collect()
    }
}
