// MARK: Docs

#[cfg(doc)]
/// Runs the given block, either returning a value borrowing data from `$ptr`'s target, or
/// continuing by evaluating to a non-borrowing value.
///
#[doc = include_str!("./maybe_borrow.md")]
#[macro_export]
macro_rules! maybe_borrow {
    ( $(for<$lt:lifetime>)? |$($ptr:ident),*| -> $Ret:ty $block:block ) => {
        todo!()
    };
}

#[cfg(doc)]
#[doc = include_str!("./try_maybe_borrow.md")]
#[macro_export]
macro_rules! try_maybe_borrow {
    ( $(for<$lt:lifetime>)? |$($ptr:ident),*| -> $Ret:ty $block:block ) => {
        todo!()
    };
}

#[cfg(doc)]
#[macro_export]
/// Return from the containing function with potentially borrowed data from within a
/// [`maybe_borrow`] or [`try_maybe_borrow`] invocation.
///
/// This macro is only available within the aforementioned macros.
macro_rules! return_borrowed {
    ($return_value:expr) => {};
}

#[cfg(doc)]
#[macro_export]
/// Behaves like [`core::task::ready!`], but uses [`return_borrowed!`] rather than `return` so it can be used within [`maybe_borrow!`] or [`try_maybe_borrow!`].
///
/// This macro is only available within the aforementioned macros.
macro_rules! ready {
    ($return_value:expr) => {};
}

#[cfg(doc)]
pub use return_borrowed;

// MARK: Public

/// Runs the given block, either returning a value borrowing data from `$ptr`'s target, or
/// continuing by evaluating to a non-borrowing value.
///
#[cfg(not(doc))]
#[doc = include_str!("./maybe_borrow.md")]
#[macro_export]
macro_rules! maybe_borrow {
    ($(for<$lt:lifetime $(,)?>)? |$($ptr:ident),+ $(,)?| -> $Ret:ty $block:block $(,)?) => {{
        $crate::_m::__maybe_borrow! {
            $crate::_m::WithLt![$($lt ->)? $Ret],
            |[$($ptr)+]| { $crate::_m::ControlFlow::Continue({
                #[allow(unused)]
                use $crate::_m::__return_borrowed as return_borrowed;
                #[allow(unused)]
                use $crate::_m::__ready as ready;

                $block
            }) }
        }
    }};

    ($(for<$lt:lifetime $(,)?>)? |$ptr:ident $(,)?| $block:expr $(,)?) => {
        $crate::_m::compile_error!("Explicit return type required in maybe_borrow!");
    }

}

pub use maybe_borrow;

#[cfg(not(doc))]
#[doc = include_str!("./try_maybe_borrow.md")]
#[macro_export]
macro_rules! try_maybe_borrow {
    ($(for<$lt:lifetime $(,)?>)? |$($ptr:ident),+ $(,)?| -> $Ret:ty $block:block $(,)?) => {
        $crate::_m::__maybe_borrow! {
            $crate::_m::WithLt![$($lt ->)? $Ret],
            |[$($ptr)+]| { $crate::_m::try_maybe_borrow_helper(|w| w.wrap({
                #[allow(unused)]
                use $crate::_m::__return_borrowed_try as return_borrowed;
                #[allow(unused)]
                use $crate::_m::__ready as ready;
                $block
            })) }
        }
    };

    ($(for<$lt:lifetime $(,)?>)? |$ptr:ident $(,)?| $block:expr $(,)?) => {
        $crate::_m::compile_error!("Explicit return type required in try_maybe_borrow!");
    }
}

pub use try_maybe_borrow;

// MARK: Internal

#[doc(hidden)]
#[macro_export]
macro_rules! __maybe_borrow {
    ($Ret:ty, |$ptr:tt| $block:block) => {{
        let (_out, _ptr) = match $crate::_m::maybe_borrow::<_, $Ret, _>(
            $crate::_m::__nest_pattern!(@input <- $ptr),
            |$crate::_m::__nest_pattern!(@mut <- $ptr), _| {
                let _ = $crate::_m::__nest_pattern!(@noop_use_mut <- $ptr);
                $block
            },
        ) {
            $crate::_m::ControlFlow::Break(_ret) => return _ret,
            $crate::_m::ControlFlow::Continue(_pair) => _pair
        };

        #[allow(unused_assignments)]
        { $crate::_m::__pointer_assign! { $ptr <- _ptr } }
        _out
    }};
}

pub use __maybe_borrow;

#[doc(hidden)]
#[macro_export]
macro_rules! __nest_pattern {
    (@$type:tt <- [$arg0:tt $($arg:tt)+]) => {
        (
            $crate::_m::__nest_pattern!(@$type <- [$arg0]),
            $crate::_m::__nest_pattern!(@$type <- [$($arg)+]),
        )
    };
    (@input <- [$arg:tt]) => { $arg };
    (@mut <- [$arg:tt]) => { mut $arg };
    (@noop_use_mut <- [$arg:tt]) => { $crate::_m::noop_use_mut(&mut $arg) };
    (@type:tt <- []) => { () };
}

pub use __nest_pattern;

#[doc(hidden)]
#[macro_export]
macro_rules! __pointer_assign {
    ([$ptr:ident] <- $value:expr) => {
        $ptr = $value;
    };
    ([$ptr0:ident $($ptr:ident)+] <- $value:expr) => {
        $ptr0 = $value.0;
        $crate::_m::__pointer_assign! { [$($ptr)*] <- $value.1 }
    };
    ([] <- $value:expr) => {
        () = $value;
    }
}

pub use __pointer_assign;

#[doc(hidden)]
#[macro_export]
macro_rules! __return_borrowed {
    ($value:expr $(,)?) => {
        return $crate::_m::ControlFlow::Break($value)
    };
}

pub use __return_borrowed;

#[doc(hidden)]
#[macro_export]
macro_rules! __return_borrowed_try {
    ($value:expr $(,)?) => {
        return $crate::_m::CustomTry::map_continue($value, $crate::_m::Break)
    };
}

pub use __return_borrowed_try;

#[doc(hidden)]
#[macro_export]
macro_rules! __ready {
    ($value:expr $(,)?) => {
        match $value {
            $crate::_m::Poll::Ready(_value) => _value,
            $crate::_m::Poll::Pending => return_borrowed!($crate::_m::Poll::Pending),
        }
    };
}

pub use __ready;
