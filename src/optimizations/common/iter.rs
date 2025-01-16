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
    fn filter<F>(self, func: F) -> Vec<T>
    where
        F: Fn(&T) -> bool;

    fn map<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> T2,
        B: FromIterator<T2>;

    fn filter_map<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> Option<T2>,
        B: FromIterator<T2>;
}

impl<I: IntoIterator<Item = T>, T> EasyIter<T> for I {
    fn filter<F>(self, func: F) -> Vec<T>
    where
        F: Fn(&T) -> bool,
    {
        std::iter::Iterator::filter(self.into_iter(), func).collect()
    }

    fn filter_map<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> Option<T2>,
        B: FromIterator<T2>,
    {
        std::iter::Iterator::filter_map(self.into_iter(), func).collect()
    }

    fn map<F, T2, B>(self, func: F) -> B
    where
        F: Fn(T) -> T2,
        B: FromIterator<T2>,
    {
        std::iter::Iterator::map(self.into_iter(), func).collect()
    }
}
