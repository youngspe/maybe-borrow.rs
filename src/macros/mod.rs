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
    ($(for<$($lt:lifetime),* $(,)?>)? |$($ptr:ident),+ $(,)?| -> $Ret:ty $block:block $(,)?) => {{
        $crate::_m::__maybe_borrow! {
            $Ret,
            [$($($lt)*)?],
            |[$($ptr)+]| { $crate::_m::ControlFlow::Continue({
                $crate::_m::__import_contextual_macros! { __return_borrowed, $block }
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
            $Ret,
            [$($lt)?],
            |[$($ptr)+]| { $crate::_m::try_maybe_borrow_helper(|w| w.wrap(
                $crate::_m::__import_contextual_macros! { __return_borrowed_try, $block }
            )) }
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
macro_rules! __actual_combined_with_lt {
    (=> $Ret:ty) => { $Ret };
    ($lt0:lifetime $($lt:lifetime)* => $Ret:ty) => {
        $crate::_m::Actual<
            $crate::_m::WithLt![
                $lt0 -> $crate::_m::__actual_combined_with_lt![$($lt)* => $Ret]
            ]
        >
    };
}

pub use __actual_combined_with_lt;

#[doc(hidden)]
#[macro_export]
macro_rules! __import_contextual_macros {
    ($return_borrowed:ident, $block:expr) => {{
        #[allow(unused)]
        use $crate::_m::__ready as ready;
        #[allow(unused)]
        use $crate::_m::$return_borrowed as return_borrowed;
        $block
    }};
}

pub use __import_contextual_macros;

#[doc(hidden)]
#[macro_export]
macro_rules! __maybe_borrow {
    ($Ret:ty, $lt:tt, |$ptr:tt| $block:block) => {{
        let _pairs = match $crate::_m::__maybe_borrow_nested! {
            $Ret, [], $lt, [], |$ptr| $block
        } {
            $crate::_m::ControlFlow::Break(_ret) => return _ret,
            $crate::_m::ControlFlow::Continue(_pairs) => _pairs,
        };

        let _out;

        #[allow(unused_assignments)]
        {
            $crate::_m::__pointer_assign! { _out $lt $ptr <- _pairs }
        }
        _out
    }};
}

pub use __maybe_borrow;

#[doc(hidden)]
#[macro_export]
macro_rules! __maybe_borrow_nested {
    // No arguments remaining:
    ($Ret:ty, $past_lt:tt, $lt:tt, [$($all_ptrs:tt)*], |[]| $block:block) => {{
        $(
            let mut $all_ptrs = $all_ptrs.0;
            $crate::_m::noop_use_mut(&mut $all_ptrs);
        )*
        $block
    }};

    // Only one lifetime parameter remaining:
    (
        $Ret:ty,
        [$($past_lt:lifetime)*], [$($lt0:lifetime)?],
        [$($past_ptrs:tt)*], |$ptr:tt| $block:block
    ) => {
            $crate::_m::maybe_borrow::<
            _,
            $crate::_m::WithLt![$($lt0 ->)? $crate::_m::__actual_combined_with_lt![
                $($past_lt)* => $Ret
            ]],
            _,
        >(
            $crate::_m::__nest_pattern!(@input <- $ptr),
            |$crate::_m::__nest_pattern!(@mut <- $ptr), _| {
                $(
                    let mut $past_ptrs = $past_ptrs.0;
                    $crate::_m::noop_use_mut(&mut $past_ptrs);
                )*
                let _ = $crate::_m::__nest_pattern!(@noop_use_mut <- $ptr);
                $block
            },
        )
    };

    (
        $Ret:ty,
        [$($past_lt:lifetime)*], [$lt0:lifetime $($lt:lifetime)+],
        [$($past_ptrs:tt)*], |[$ptr0:ident $($ptr:ident)*]| $block:block
    ) => {
            $crate::_m::maybe_borrow::<
            _,
            $crate::_m::WithLt![$lt0 -> $crate::_m::__actual_combined_with_lt![
                $($past_lt)* $($lt)* => $Ret
            ]],
            _,
        >(
            $ptr0,
            |$ptr0, _| {
                let $ptr0 = $crate::_m::ForceMove($ptr0);
                $crate::_m::__maybe_borrow_nested! {
                    $Ret,
                    [$($past_lt)* $lt0], [$($lt)*],
                    [$($past_ptrs)* $ptr0], |[ $($ptr)* ]| $block
                }
            },
        )
    };
}

pub use __maybe_borrow_nested;

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
    ( @final [$ptr0:ident $($ptr:ident)+] <- $value:expr) => {
        let _value = $value;
        $ptr0 = _value.0;
        $crate::_m::__pointer_assign! { @final [$($ptr)*] <- _value.1 }
    };
    ( @final [$ptr0:ident] <- $value:expr) => {
        $ptr0 = $value;
    };
    ( @final [] <- $value:expr) => {
        () = $value;
    };
    ($out:ident [$($lt:lifetime)?] $ptr:tt <- $value:expr) => {
        let (_out, _value) = $value;
        $out = _out;
        $crate::_m::__pointer_assign! { @final $ptr <- _value }
    };
    ($out:ident [$lt0:lifetime $($lt:lifetime)+] [$ptr0:ident $($ptr:ident)*] <- $value:expr) => {
        let _value = $value;
        $ptr0 = _value.1;
        $crate::_m::__pointer_assign! { $out [$($lt)*] [$($ptr)*] <- _value.0 }
    };
    ($out:ident $lt:tt [] <- $value:expr) => {
        compile_error!(stringify!($out $lt $x));
        $out = $value;
    };
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
