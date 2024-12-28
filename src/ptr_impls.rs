use core::{ops::Deref, pin::Pin};

use crate::{maybe_borrow_impl::BorrowedAs, traits::Reborrow, utils::ref_cast_mut, WithLt};

unsafe impl<'ptr, T: ?Sized> Reborrow<'ptr> for &'ptr T {
    type BorrowWithLifetime = WithLt!['b -> &'b T];

    fn reborrow<'b>(this: &'b mut &'ptr T) -> &'b T {
        this
    }
    unsafe fn extend<'b>(this: &'ptr T) -> &'b T {
        unsafe { &*(this as *const T) }
    }
}

unsafe impl<'ptr, T: ?Sized> Reborrow<'ptr> for &'ptr mut T {
    type BorrowWithLifetime = WithLt!['b -> &'b mut T];

    fn reborrow<'b>(this: &'b mut &'ptr mut T) -> &'b mut T {
        this
    }
    unsafe fn extend<'b>(this: &'ptr mut T) -> &'b mut T {
        unsafe { &mut *(this as *mut T) }
    }
}

unsafe impl<'ptr, Ptr> Reborrow<'ptr> for Pin<Ptr>
where
    Ptr: Reborrow<'ptr> + Deref,
    for<'b> BorrowedAs<'b, Ptr::BorrowWithLifetime>: Deref,
{
    type BorrowWithLifetime = WithLt!['b -> Pin<BorrowedAs<'b, Ptr::BorrowWithLifetime>>];

    fn reborrow<'b>(this: &'b mut Pin<Ptr>) -> Pin<BorrowedAs<'b, Ptr::BorrowWithLifetime>> {
        unsafe {
            // SAFETY: Pin is repr(transparent), so transmuting from &mut Pin<Ptr> to &mut Ptr
            // should be valid.
            let inner: &mut Ptr = ref_cast_mut(this);
            Pin::new_unchecked(Ptr::reborrow(inner))
        }
    }

    unsafe fn extend<'b>(this: Pin<Ptr>) -> Pin<BorrowedAs<'b, Ptr::BorrowWithLifetime>> {
        unsafe {
            // SAFETY: Ptr::extend yields the same pointer with a different lifetime, and nothing
            // pinned is being moved.
            let inner = Pin::into_inner_unchecked(this);
            Pin::new_unchecked(Ptr::extend(inner))
        }
    }
}

unsafe impl<'ptr, P1, P2> Reborrow<'ptr> for (P1, P2)
where
    P1: Reborrow<'ptr>,
    P2: Reborrow<'ptr>,
{
    type BorrowWithLifetime = WithLt!['b -> (
        BorrowedAs<'b, P1::BorrowWithLifetime>,
        BorrowedAs<'b, P2::BorrowWithLifetime>,
    )];

    fn reborrow<'b>(
        (p1, p2): &'b mut Self,
    ) -> (
        BorrowedAs<'b, P1::BorrowWithLifetime>,
        BorrowedAs<'b, P2::BorrowWithLifetime>,
    ) {
        (P1::reborrow(p1), P2::reborrow(p2))
    }

    unsafe fn extend<'b>(
        (p1, p2): Self,
    ) -> (
        BorrowedAs<'b, P1::BorrowWithLifetime>,
        BorrowedAs<'b, P2::BorrowWithLifetime>,
    ) {
        unsafe { (P1::extend(p1), P2::extend(p2)) }
    }
}
