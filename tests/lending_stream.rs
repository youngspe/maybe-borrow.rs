use std::{
    pin::{pin, Pin},
    task::{ready, Context, Poll},
};

use futures::{executor::block_on, prelude::*};

use pin_project_lite::pin_project;

use maybe_borrow::prelude::*;

trait LendingStreamBase<'iter, _Bound = &'iter Self>: 'iter {
    type Item: 'iter;
    fn poll_next_base(self: Pin<&'iter mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>>;
}

trait LendingStreamLt<'iter>: 'iter {
    type ItemLt: 'iter;
    fn poll_next_lt(self: Pin<&'iter mut Self>, cx: &mut Context) -> Poll<Option<Self::ItemLt>>;
}

impl<'iter, I: ?Sized + LendingStreamBase<'iter>> LendingStreamLt<'iter> for I {
    type ItemLt = I::Item;
    fn poll_next_lt(self: Pin<&'iter mut Self>, cx: &mut Context) -> Poll<Option<Self::ItemLt>> {
        self.poll_next_base(cx)
    }
}

trait LendingStream: for<'iter> LendingStreamLt<'iter, ItemLt = Self::Item<'iter>> {
    type Item<'iter>;

    fn poll_next<'iter>(
        self: Pin<&'iter mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<<Self as LendingStreamLt<'iter>>::ItemLt>> {
        self.poll_next_lt(cx)
    }
}

impl<I: ?Sized + for<'iter> LendingStreamLt<'iter>> LendingStream for I {
    type Item<'iter> = <Self as LendingStreamLt<'iter>>::ItemLt;
}

pin_project!(
    struct Windows<T, I> {
        items: Vec<T>,
        size: usize,
        #[pin]
        iter: I,
    }
);

impl<T, I> Windows<T, I> {
    pub fn new(iter: I, size: usize) -> Self {
        let iter = iter;
        Self {
            items: Vec::with_capacity(size),
            size,
            iter,
        }
    }
}

impl<'iter, T, I> LendingStreamBase<'iter> for Windows<T, I>
where
    I: Stream<Item = T>,
{
    type Item = &'iter mut [T];
    fn poll_next_base(
        self: Pin<&'iter mut Self>,
        cx: &mut Context,
    ) -> Poll<Option<&'iter mut [T]>> {
        let mut this = self.project();

        while let Some(item) = ready!(this.iter.as_mut().poll_next(cx)) {
            if this.items.len() < *this.size {
                this.items.push(item);

                if this.items.len() < *this.size {
                    continue;
                }
            } else if let Some(slot) = this.items.first_mut() {
                *slot = item;
                this.items.rotate_left(1);
            }

            return Poll::Ready(Some(&mut this.items[..]));
        }

        this.items.clear();
        Poll::Ready(None)
    }
}

fn poll_next_filtered<'iter, I: LendingStream>(
    mut iter: Pin<&'iter mut I>,
    mut predicate: impl FnMut(&I::Item<'_>) -> bool,
    cx: &mut Context,
) -> Poll<Option<I::Item<'iter>>> {
    loop {
        maybe_borrow!(for<'x> |iter| -> Poll<Option<I::Item<'x>>> {
            match ready!(iter.poll_next(cx)) {
                Some(ref item) if !predicate(item) => {}
                out => return_borrowed!(Poll::Ready(out)),
            }
        });
    }
}

fn poll_next_filtered_with_try<'iter, I: LendingStream>(
    mut iter: Pin<&'iter mut I>,
    mut predicate: impl FnMut(&I::Item<'_>) -> bool,
    cx: &mut Context,
) -> Poll<Option<I::Item<'iter>>> {
    loop {
        maybe_borrow!(for<'x> |iter| -> Poll<Option<I::Item<'x>>> {
            match ready!(iter.poll_next(cx)) {
                Some(ref item) if !predicate(item) => {}
                out => return_borrowed!(Poll::Ready(out)),
            }
        });
    }
}

#[test]
fn test_next_filtered() {
    block_on(async {
        let mut stream = pin!(Windows::new(
            stream::iter([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            3
        ));

        let items = futures::stream::poll_fn(move |cx| {
            poll_next_filtered(stream.as_mut(), |x| x[0] % 2 != 0, cx)
                .map(|x| x.map(|x| x.to_vec()))
        })
        .collect::<Vec<_>>()
        .await;

        assert_eq!(items, [[1, 2, 3], [3, 4, 5], [5, 6, 7], [7, 8, 9]])
    });
}

#[test]
fn test_next_filtered_with_try() {
    block_on(async {
        let mut stream = pin!(Windows::new(
            stream::iter([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]),
            3
        ));

        let items = futures::stream::poll_fn(move |cx| {
            poll_next_filtered_with_try(stream.as_mut(), |x| x[0] % 2 != 0, cx)
                .map(|x| x.map(|x| x.to_vec()))
        })
        .collect::<Vec<_>>()
        .await;

        assert_eq!(items, [[1, 2, 3], [3, 4, 5], [5, 6, 7], [7, 8, 9]])
    });
}
