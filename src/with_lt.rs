#[doc(hidden)]
pub use core::marker::PhantomData;

#[doc(hidden)]
pub trait Lt<'a> {
    type Value;
}

#[doc(hidden)]
pub struct _With<W: ?Sized>(PhantomData<W>);

impl<'lt, F: ?Sized, T> Lt<'lt> for F
where
    F: Fn(&'lt ()) -> PhantomData<(&'lt (), T)>,
{
    type Value = T;
}

pub trait WithLtHrtb<'a> {
    type ActualLt;
}

pub trait WithLt: for<'a> WithLtHrtb<'a, ActualLt = Self::Actual<'a>> {
    type Actual<'a>;
}

#[doc(hidden)]
impl<'a, L: ?Sized + Lt<'a>> WithLtHrtb<'a> for _With<L> {
    type ActualLt = L::Value;
}

#[doc(hidden)]
impl<L: ?Sized + for<'a> Lt<'a>> WithLt for _With<L> {
    type Actual<'a> = <L as Lt<'a>>::Value;
}

#[doc(hidden)]
pub type Actual<'a, W> = <W as WithLtHrtb<'a>>::ActualLt;

mod macros {
    #[doc(hidden)]
    #[macro_export]
    macro_rules! WithLt {
        ($lt:lifetime -> $Ty:ty) => {
            $crate::_m::_With<dyn for<$lt> $crate::_m::Lt<$lt, Value = $Ty>>
        };
        ($Ty:ty) => {
            $crate::_m::_With<
                fn(&()) -> $crate::_m::PhantomData<(&(), $Ty)>
            >
        };
    }
    pub use WithLt;
}

pub use macros::WithLt;
