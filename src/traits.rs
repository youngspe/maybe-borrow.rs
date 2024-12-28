use crate::{maybe_borrow_impl::BorrowedAs, with_lt::WithLt};

pub trait BorrowWithLifetime<'b> {
    type Pointer: Reborrow<'b, BorrowWithLifetime = Self>;
}

impl<'b, W> BorrowWithLifetime<'b> for W
where
    W: WithLt,
    W::Actual<'b>: Reborrow<'b, BorrowWithLifetime = W>,
{
    type Pointer = W::Actual<'b>;
}

pub unsafe trait Reborrow<'ptr> {
    type BorrowWithLifetime: ?Sized + 'ptr + for<'b> BorrowWithLifetime<'b>;
    fn reborrow<'b>(this: &'b mut Self) -> BorrowedAs<'b, Self::BorrowWithLifetime>;
    unsafe fn extend<'b>(this: Self) -> BorrowedAs<'b, Self::BorrowWithLifetime>;
}
