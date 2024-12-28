As a hypothetical demonstration:

```rust ignore
fn get_data_or_initialize(mut borrow: &mut SomeType) -> BorrowedData<'x> {
    maybe_borrow!(for<'x> |borrow| -> BorrowedData<'x> {
        let data: BorrowedData = borrow.get_borrowed_data_mut();

        // Return the borrowed data only if it's initialized
        if (data.is_initialized()) {
            return_borrowed!(data);
        }
    });

    // `borrow` is still accessible because its data did not escape the above
    // block.
    borrow.initialize();
    borrow.get_borrowed_data_mut()
}
```

## Control flow

This macro internally places `$block` inside a closure, so returning or breaking from within will not work as expected.

* The <dfn>[`return_borrowed!`]</dfn> macro is used to return from the function that contains the `maybe_borrow!` invocation.
* If `return_borrowed!` is not used and the block exits normally, it will evaluate to the trailing expression like any block. The trailing expression cannot reference `$ptr`.
* If the macro completes without returning, the variable referenced by `$ptr` is still fully accessible because no borrowed data was allowed to escape the block.

## Examples

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

