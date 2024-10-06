use std::{cmp::max, marker::PhantomData, num::NonZeroUsize};

use crate::Dataset;

/// Functionality to create windows.
pub trait Windows<I> {
    /// Returns an iterator over all the windows of length `size`. The windows overlap.
    ///
    /// # Returns
    ///
    /// A `WindowsIterator` instance.
    fn windows(&self, size: usize) -> WindowsIterator<'_, I>;
}

impl<I, T: Dataset<I>> Windows<I> for T {
    /// Is empty if the `Dataset` is shorter than `size`.
    ///
    /// # Panics
    ///
    /// Panics if `size` is 0.    
    ///
    /// # Examples
    ///
    /// ```
    /// use crate::{
    ///    transform::{Windows, WindowsDataset},
    ///    Dataset, InMemDataset,
    /// };
    ///
    /// let items = [1, 2, 3, 4].to_vec();
    /// let dataset = InMemDataset::new(items.clone());
    ///
    /// for window in dataset.windows(2) {
    ///  // do sth with window
    /// }
    /// ```
    fn windows(&self, size: usize) -> WindowsIterator<'_, I> {
        let size = NonZeroUsize::new(size).expect("window size must be non-zero");
        WindowsIterator::new(self, size)
    }
}

/// Overlapping windows iterator.
pub struct WindowsIterator<'a, I> {
    /// The size of the windows.
    pub size: NonZeroUsize,
    current: usize,
    dataset: &'a dyn Dataset<I>,
}

impl<'a, I> WindowsIterator<'a, I> {
    /// Creates a new `WindowsIterator` instance. The windows overlap.
    /// Is empty if the input `Dataset` is shorter than `size`.
    ///
    /// # Parameters
    ///
    /// - `dataset`: The dataset over which windows will be created.
    /// - `size`: The size of the windows.
    ///
    /// # Returns
    ///
    /// A `Windows` iterator.
    pub fn new(dataset: &'a dyn Dataset<I>, size: NonZeroUsize) -> Self {
        WindowsIterator {
            current: 0,
            dataset,
            size,
        }
    }
}

impl<'a, I> Iterator for WindowsIterator<'a, I> {
    type Item = Vec<I>;

    fn next(&mut self) -> Option<Vec<I>> {
        let items = (self.current..self.current + self.size.get())
            .map(|x| self.dataset.get(x))
            .collect();

        self.current += 1;
        items
    }
}

impl<'a, I> Clone for WindowsIterator<'a, I> {
    fn clone(&self) -> Self {
        WindowsIterator {
            size: self.size,
            dataset: self.dataset,
            current: self.current,
        }
    }
}

/// Dataset designed to work with overlapping windows of data.
pub struct WindowsDataset<D, I> {
    /// The size of the windows.
    pub size: NonZeroUsize,
    dataset: D,
    input: PhantomData<I>,
}

impl<D, I> WindowsDataset<D, I>
where
    D: Dataset<I>,
{
    /// Creates a new `WindowsDataset` instance. The windows overlap.
    /// Is empty if the input `Dataset` is shorter than `size`.
    ///
    /// # Parameters
    ///
    /// - `dataset`: The dataset over which windows will be created.
    /// - `size`: The size of the windows.
    ///
    /// # Returns
    ///
    /// A `WindowsDataset` instance.
    pub fn new(dataset: D, size: usize) -> Self
    where
        D:,
    {
        let size = NonZeroUsize::new(size).expect("window size must be non-zero");
        WindowsDataset::<D, I> {
            size,
            dataset,
            input: PhantomData,
        }
    }
}

impl<D, I> Dataset<Vec<I>> for WindowsDataset<D, I>
where
    D: Dataset<I>,
    I: Clone + Send + Sync,
{
    /// Retrieves a window of items from the dataset.
    ///
    /// # Parameters
    ///
    /// - `index`: The index of the window.
    ///
    /// # Returns
    ///
    /// A vector containing the items of the window.
    fn get(&self, index: usize) -> Option<Vec<I>> {
        (index..index + self.size.get())
            .map(|x| self.dataset.get(x))
            .collect()
    }

    /// Retrieves the number of windows in the dataset.
    ///
    /// # Returns
    ///
    /// A size representing the number of windows.
    fn len(&self) -> usize {
        let len = self.dataset.len() as isize - self.size.get() as isize + 1;
        max(len, 0) as usize
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use crate::{
        transform::{Windows, WindowsDataset},
        Dataset, InMemDataset,
    };

    #[rstest]
    pub fn windows_should_be_equal_to_vec_windows() {
        let items = [1, 2, 3, 4, 5].to_vec();
        let dataset = InMemDataset::new(items.clone());
        let expected = items
            .windows(3)
            .map(|x| x.to_vec())
            .collect::<Vec<Vec<i32>>>();

        let result = dataset.windows(3).collect::<Vec<Vec<i32>>>();

        assert_eq!(result, expected);
    }

    #[rstest]
    #[should_panic(expected = "window size must be non-zero")]
    pub fn windows_should_panic() {
        let items = [1, 2].to_vec();
        let dataset = InMemDataset::new(items.clone());

        dataset.windows(0);
    }

    #[rstest]
    #[should_panic(expected = "window size must be non-zero")]
    pub fn new_window_dataset_should_panic() {
        let items = [1, 2].to_vec();
        let dataset = InMemDataset::new(items.clone());

        WindowsDataset::new(dataset, 0);
    }

    #[rstest]
    pub fn window_dataset_len_should_be_equal() {
        let dataset = InMemDataset::new([1, 2, 3, 4].to_vec());

        let result = WindowsDataset::new(dataset, 2).len();

        assert_eq!(result, 3);
    }

    #[rstest]
    pub fn window_iterator_should_be_empty() {
        let dataset = InMemDataset::new([1, 2].to_vec());
        let mut peekable = dataset.windows(4).peekable();

        let result = peekable.peek();

        assert_eq!(result, None);
    }

    #[rstest]
    pub fn window_dataset_len_should_be_zero() {
        let dataset = InMemDataset::new([1, 2].to_vec());

        let result = WindowsDataset::new(dataset, 4).len();

        assert_eq!(result, 0);
    }

    #[rstest]
    pub fn window_dataset_get_should_be_equal() {
        let dataset = InMemDataset::new([1, 2, 3, 4].to_vec());
        let expected = Some([1, 2, 3].to_vec());

        let result = WindowsDataset::new(dataset, 3).get(0);

        assert_eq!(result, expected);
    }

    #[rstest]
    pub fn window_dataset_get_should_be_none() {
        let dataset = InMemDataset::new([1, 2].to_vec());

        let result = WindowsDataset::new(dataset, 4).get(0);

        assert_eq!(result, None);
    }
}
