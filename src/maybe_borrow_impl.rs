use core::{marker::PhantomData, mem::ManuallyDrop, ops::ControlFlow};

use crate::{
    traits::*,
    with_lt::{Actual, WithLt},
};

pub type BorrowedAs<'b, B> = <B as BorrowWithLifetime<'b>>::Pointer;

pub fn maybe_borrow<'ptr, Ptr: 'ptr + Reborrow<'ptr>, B: WithLt, C>(
    this: Ptr,
    block: impl for<'unknown> FnOnce(
        BorrowedAs<'unknown, Ptr::BorrowWithLifetime>,
        PhantomData<&'unknown ()>,
    ) -> ControlFlow<Actual<'unknown, B>, C>,
) -> ControlFlow<Actual<'ptr, B>, (C, Ptr)> {
    let mut this = ManuallyDrop::new(this);

    let ctrl = {
        let erased_borrow = unsafe { Reborrow::extend(Ptr::reborrow(&mut *this)) };

        block(erased_borrow, PhantomData)
    };

    match ctrl {
        ControlFlow::Break(out) => ControlFlow::Break(out),
        ControlFlow::Continue(out) => ControlFlow::Continue((out, ManuallyDrop::into_inner(this))),
    }
}
