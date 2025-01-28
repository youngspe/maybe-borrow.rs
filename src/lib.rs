//! [`maybe_borrow!`]: maybe_borrow
//! [`try_maybe_borrow!`]: try_maybe_borrow
#![doc = include_str!("../README.md")]
#![no_std]

extern crate should_it_compile;

mod custom_try;
mod macros;
mod ptr_impls;
mod utils;
mod with_lt;

should_it_compile::compile_test_mod!(compile_fail);

pub mod prelude {
    pub use crate::macros::{maybe_borrow, try_maybe_borrow};
}

#[doc(hidden)]
pub mod _m {
    use crate::custom_try::TryKind;
    pub use crate::{
        custom_try::{ContinueOf, CustomTry, WithContinue},
        macros::*,
        maybe_borrow_impl::maybe_borrow,
        with_lt::*,
    };
    pub use core::{
        self,
        marker::PhantomData,
        ops::ControlFlow::{self, Break, Continue},
        prelude::rust_2021::*,
        result::Result::{Err, Ok},
        task::Poll,
    };

    /// Container that's not [`Copy`] so it automatically gets moved into a closure
    /// rather than referenced.
    pub struct ForceMove<T>(pub T);

    /// Does nothing with a mutable borrow of `T`.
    /// Call this from a macro to prevent `unused_mut` by proving a variable is mutably borrowed at
    /// least once.
    #[inline(always)]
    pub fn noop_use_mut<T: ?Sized>(_: &mut T) {}

    pub struct WrapTryMaybeBorrowExit<Out>(PhantomData<Out>);

    impl<Out, Exit, T> WrapTryMaybeBorrowExit<Out>
    where
        Out: CustomTry<Continue = ControlFlow<T, Exit>>,
    {
        pub fn wrap(self, exit: Exit) -> Out {
            Out::from_continue(Continue(exit))
        }
    }

    pub fn try_maybe_borrow_helper<Tk, Ret, T, Exit>(
        body: impl FnOnce(
            WrapTryMaybeBorrowExit<Tk::WithContinue<ControlFlow<T, Exit>>>,
        ) -> Tk::WithContinue<ControlFlow<T, Exit>>,
    ) -> ControlFlow<Ret, Exit>
    where
        Tk: TryKind,
        Ret: CustomTry<Kind = Tk, Continue = T>,
    {
        let body_out = body(WrapTryMaybeBorrowExit(PhantomData));
        match body_out.into_ctrl() {
            Break(r) => Break(Ret::from_residual(r)),
            Continue(Ok(Break(b))) => Break(Ret::from_continue(b)),
            Continue(Ok(Continue(exit))) => Continue(exit),
            Continue(Err(e)) => Break(Ret::from_empty(e)),
        }
    }
}

mod traits;

mod maybe_borrow_impl;
