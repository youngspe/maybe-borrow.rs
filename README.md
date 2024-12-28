This crate provides the [`maybe_borrow!`] and [`try_maybe_borrow!`] macros for conditionally returning borrowed data, without losing access to the original borrow in the non-returning case.


## Preface

This crate is **extremely** heavily inspired by
[`polonius-the-crab`](https://docs.rs/polonius-the-crab/latest/polonius_the_crab/) by
[@danielhenrymantilla](https://github.com/danielhenrymantilla),
so please check that out!

## Motivation

A current borrow checker limitation assumes that when borrowed data is conditionally returned, that data is considered borrowed for the lifetime of the function.

Following are examples of code that are affected by this limitation.

### Case: Downcast fallback

Let's say you have `&mut dyn Any` reference that might point to a `Vec<T>` or `Box<[T]>`.
If it does refer to a value of one of those types, you should be able to a `&mut [T]` reference to its data. This following code looks like it should do the trick:


```rust compile_fail
use std::any::Any;

fn downcast_slice_mut<T: 'static>(src: &mut dyn Any) -> Result<&mut [T], &mut dyn Any> {
    if let Some(v) = src.downcast_mut::<Vec<T>>() {
        return Ok(&mut *v)
    }

    if let Some(b) = src.downcast_mut::<Box<[T]>>() {
        return Ok(&mut *b)
    }

    return Err(src)
}
```

<details><summary>

#### Error diagnostic
</summary>

The above code produces the following errors:

```plain
error[E0499]: cannot borrow `*src` as mutable more than once at a time
  --> src\lib.rs:10:22
   |
5  | fn downcast_slice_mut<T: 'static>(src: &mut dyn Any) -> Result<&mut [T], &mut dyn Any> {
   |                                        - let's call the lifetime of this reference `'1`
6  |     if let Some(v) = src.downcast_mut::<Vec<T>>() {
   |                      --- first mutable borrow occurs here
7  |         return Ok(&mut *v)
   |                ----------- returning this value requires that `*src` is borrowed for `'1`
...
10 |     if let Some(b) = src.downcast_mut::<Box<[T]>>() {
   |                      ^^^ second mutable borrow occurs here

error[E0499]: cannot borrow `*src` as mutable more than once at a time
  --> src\lib.rs:14:16
   |
5  | fn downcast_slice_mut<T: 'static>(src: &mut dyn Any) -> Result<&mut [T], &mut dyn Any> {
   |                                        - let's call the lifetime of this reference `'1`
6  |     if let Some(v) = src.downcast_mut::<Vec<T>>() {
   |                      --- first mutable borrow occurs here
7  |         return Ok(&mut *v)
   |                ----------- returning this value requires that `*src` is borrowed for `'1`
...
14 |     return Err(src)
   |                ^^^ second mutable borrow occurs here

error: aborting due to 2 previous errors
```

After attempting to downcast `src` to `Vec<T>` fails, `src` is considered borrowed for the remainder of the function because the conditional return.

</details>
<details open><summary>

#### Solution using [`maybe_borrow!`]
</summary>

We can place each conditional return in a [`maybe_borrow!`] invocation, which ensures that `src` is still usable in the event that the downcast fails and a borrowed value is not returned.

```rust
use std::any::Any;
use maybe_borrow::prelude::*;

fn downcast_slice_mut<T: 'static>(mut src: &mut dyn Any) -> Result<&mut [T], &mut dyn Any> {
    maybe_borrow!(for<'x> |src| -> Result<&'x mut [T], &'x mut dyn Any> {
        if let Some(v) = src.downcast_mut::<Vec<T>>() {
            return_borrowed!(Ok(&mut *v))
        }
    });

    maybe_borrow!(for<'x> |src| -> Result<&'x mut [T], &'x mut dyn Any> {
        if let Some(b) = src.downcast_mut::<Box<[T]>>() {
            return_borrowed!(Ok(&mut *b))
        }
    });

    return Err(src)
}
```

</details>

### Case: The lending iterator

[rust-lang/rust#92985](https://github.com/rust-lang/rust/issues/92985)


```rust compile_fail
trait LendingIterator<'iter> {
    type Item;
    fn next(&'iter mut self) -> Option<Self::Item>;
}

/// Returns the next item from `iter` that satisfies the given predicate
fn next_filtered<'iter, I: LendingIterator>(
    mut iter: &'iter mut I,
    mut predicate: impl FnMut(&I::Item<'_>) -> bool,
) -> Option<Item<'iter, I>> {
    loop {
        match iter.next() {
            Some(ref item) if !predicate(item) => {},
            out => return out,
        }
    }
}
```
<details><summary>

#### Error diagnostic
</summary>

The above code produces the following error:

```plain
error[E0499]: cannot borrow `*iter` as mutable more than once at a time
  --> src/lending_iterator.rs:15:14
   |
10 | fn next_filtered<'iter, I: for<'x> LendingIterator<'x>>(
   |                  ----- lifetime `'iter` defined here
...
15 |         match iter.next() {
   |               ^^^^ `*iter` was mutably borrowed here in the previous iteration of the loop
16 |             Some(ref item) if !predicate(item) => {},
17 |             out => return out,
   |                           --- returning this value requires that `*iter` is borrowed for `'iter`
```

</details>
<details open><summary>

#### Solution with [`maybe_borrow!`] or [`try_maybe_borrow!`]
</summary>

```rust
use maybe_borrow::prelude::*;

trait LendingIterator {
    type Item<'iter>: 'iter where Self: 'iter;
    fn next(&mut self) -> Option<Self::Item<'_>>;
}
type Item<'iter, I> = <I as LendingIterator>::Item<'iter>;


/// Returns the next item from `iter` that satisfies the given predicate
fn next_filtered<'iter, I: LendingIterator>(
    mut iter: &'iter mut I,
    mut predicate: impl FnMut(&Item<I>) -> bool,
) -> Option<Item<'iter, I>> {
    loop {
        maybe_borrow!(for<'x> |iter| -> Option<Item<'x, I>> {
            match iter.next() {
                Some(ref item) if !predicate(item) => {},
                out => return_borrowed!(out),
            }
        });
    }
}

fn next_filtered_with_try<'iter, I: LendingIterator>(
    mut iter: &'iter mut I,
    mut predicate: impl FnMut(&Item<I>) -> bool,
) -> Option<Item<'iter, I>> {
    loop {
        try_maybe_borrow!(for<'x> |iter| -> Option<Item<'x, I>> {
            let item = iter.next()?;
            if predicate(&item) {
                return_borrowed!(Some(item));
            }
        });
    }
}
```

</details>

## Working with pinned data

The [`maybe_borrow!`] and [`try_maybe_borrow!`] macros work on <code>[Pin]\<&mut T></code> references in addition to plain `&mut T` references.

```rust
use maybe_borrow::prelude::*;

trait LendingStream {
    type Item<'iter> where Self: 'iter;
    fn next<'iter>(&'iter mut self) -> Option<Self::Item<'iter>>;
}

/// Returns the next item from `iter` that satisfies the given predicate
fn poll_next_filtered<'iter, I: LendingStream>(
    mut iter: &'iter mut I,
    mut predicate: impl FnMut(&I::Item<'_>) -> bool,
) -> Option<I::Item<'iter>> {
    loop {
        maybe_borrow!(for<'x> |iter| -> Option<I::Item<'x>> {
            match iter.next() {
                Some(ref item) if !predicate(item) => {},
                out => return_borrowed!(out),
            }
        });
    }
}
```

## Operating on multiple references

Multiple references can be used as long as they have the same lifetime:

```rust
use std::{borrow::Borrow, collections::HashMap};
use maybe_borrow::maybe_borrow;

/// Finds the first key in `keys` that can be found in `map` and returns a mutable reference to its
/// value, or `None` if none of the keys exist in `map`.
fn get_first_available_mut<T>(
    mut map: &mut HashMap<String, T>,
    keys: impl IntoIterator<Item: Borrow<str>>,
) -> Option<&mut T> {
    for key in keys {
        maybe_borrow!(for<'x> |map| -> Option<&'x mut T> {
            if let value @ Some(_) = map.get_mut(key.borrow()) {
                // Use `return_borrowed!()` to return with borrowed data.
                return_borrowed!(value);
            }
        });
    }

    None
}
```

## Notes

As mentioned above, this crate is largely based on
[`polonius-the-crab`](https://docs.rs/polonius-the-crab/latest/polonius_the_crab/).
If you don't need features like pinned references or multiple references, `polonius-the-crab` is better documented, better tested, and likely the better choice.

[`maybe_borrow!`]: #
[`try_maybe_borrow!`]: #
[Pin]: https://doc.rust-lang.org/std/pin/struct.Pin.html
