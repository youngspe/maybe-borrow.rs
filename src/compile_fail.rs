use should_it_compile::should_not_compile;

should_not_compile!({
    prefix!({
        use maybe_borrow::prelude::*;
    });

    mod pairs {
        fn distinct_lifetimes() {
            // should fail because a and b have different lifetimes
            fn inner(mut a: &mut i32, mut b: &mut i32) {
                maybe_borrow!(for<'x> |a, b| -> () {
                    if a == b {
                        return_borrowed!(());
                    }
                })
            }
        }

        fn lifetimes_with_common_supertype() {
            // should fail because 'a and 'b are distinct despite both outliving 'c
            fn inner<'a: 'c, 'b: 'c, 'c>(mut a: &'a mut i32, mut b: &'b mut i32) {
                maybe_borrow!(for<'x> |a, b| -> () {
                    if (*a < *b) {
                        return_borrowed!(())
                    }
                })
            }
        }

        fn lifetimes_with_common_supertype_and_borrowed_return() {
            // should fail because 'a and 'b are distinct despite both outliving 'c
            fn inner<'a: 'c, 'b: 'c, 'c>(mut a: &'a mut i32, mut b: &'b mut i32) -> &'c mut i32 {
                maybe_borrow!(for<'x> |a, b| -> &'x mut i32 {
                    if (*a < *b) {
                        return_borrowed!(a)
                    }
                });
                b
            }
        }
    }
});
