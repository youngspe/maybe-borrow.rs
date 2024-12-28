use maybe_borrow::prelude::*;

trait LendingIteratorBase<'iter, _Bound = &'iter Self>: 'iter {
    type Item: 'iter;
    fn next_base(&'iter mut self) -> Option<Self::Item>;
}

trait LendingIteratorLt<'iter>: 'iter {
    type ItemLt: 'iter;
    fn next_lt(&'iter mut self) -> Option<Self::ItemLt>;
}

impl<'iter, I: ?Sized + LendingIteratorBase<'iter>> LendingIteratorLt<'iter> for I {
    type ItemLt = I::Item;
    fn next_lt(&'iter mut self) -> Option<Self::ItemLt> {
        I::next_base(self)
    }
}

trait LendingIterator: for<'iter> LendingIteratorLt<'iter, ItemLt = Self::Item<'iter>> {
    type Item<'iter>
    where
        Self: 'iter;

    fn next<'iter>(&'iter mut self) -> Option<Self::Item<'iter>> {
        self.next_lt()
    }
}

impl<I: ?Sized + for<'iter> LendingIteratorLt<'iter>> LendingIterator for I {
    type Item<'iter> = <Self as LendingIteratorLt<'iter>>::ItemLt;
}

enum WindowsState<T> {
    Created { size: usize },
    Initialized { data: Box<[T]> },
}

struct Windows<T, I> {
    state: WindowsState<T>,
    iter: I,
}

impl<T, I> Windows<T, I> {
    pub fn new(iter: impl IntoIterator<IntoIter = I>, size: usize) -> Self {
        let iter = iter.into_iter();
        Self {
            state: WindowsState::Created { size },
            iter,
        }
    }
}

impl<'iter, T, I> LendingIteratorBase<'iter> for Windows<T, I>
where
    I: Iterator<Item = T>,
{
    type Item = &'iter mut [T];
    fn next_base(&'iter mut self) -> Option<&'iter mut [T]> {
        match self.state {
            WindowsState::Created { size } => {
                let data: Box<[T]> = self.iter.by_ref().take(size).collect();
                if data.len() < size {
                    return None;
                }

                self.state = WindowsState::Initialized { data };
            }
            WindowsState::Initialized { ref data } if data.len() == 0 => {
                return Some(&mut []);
            }
            WindowsState::Initialized { ref mut data } => {
                let Some(item) = self.iter.next() else {
                    let size = data.len();
                    self.state = WindowsState::Created { size };
                    return None;
                };
                data.rotate_left(1);
                *data.last_mut().unwrap() = item;
            }
        }

        let WindowsState::Initialized { ref mut data } = self.state else {
            unreachable!();
        };

        Some(data)
    }
}

fn next_filtered<'iter, I: LendingIterator>(
    mut iter: &'iter mut I,
    mut predicate: impl FnMut(&I::Item<'_>) -> bool,
) -> Option<I::Item<'iter>> {
    loop {
        maybe_borrow!(for<'x> |iter| -> Option<I::Item<'x>> {
            match iter.next() {
                Some(ref item) if !predicate(item) => {}
                out => return_borrowed!(out),
            }
        });
    }
}

fn next_filtered_with_try<'iter, I: LendingIterator>(
    mut iter: &'iter mut I,
    mut predicate: impl FnMut(&I::Item<'_>) -> bool,
) -> Option<I::Item<'iter>> {
    loop {
        try_maybe_borrow!(for<'x> |iter| -> Option<I::Item<'x>> {
            let item = iter.next()?;
            if predicate(&item) {
                return_borrowed!(Some(item));
            }
        });
    }
}

#[test]
fn test_next_filtered() {
    let mut iter = Windows::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 3);

    let items = Vec::from_iter(std::iter::from_fn(move || {
        next_filtered(&mut iter, |x| x[0] % 2 != 0).map(|x| x.iter().copied().collect::<Vec<_>>())
    }));

    assert_eq!(items, [[1, 2, 3], [3, 4, 5], [5, 6, 7], [7, 8, 9],])
}

#[test]
fn test_next_filtered_with_try() {
    let mut iter = Windows::new([1, 2, 3, 4, 5, 6, 7, 8, 9, 10], 3);

    let items = Vec::from_iter(std::iter::from_fn(move || {
        next_filtered_with_try(&mut iter, |x| x[0] % 2 != 0)
            .map(|x| x.iter().copied().collect::<Vec<_>>())
    }));

    assert_eq!(items, [[1, 2, 3], [3, 4, 5], [5, 6, 7], [7, 8, 9],])
}
