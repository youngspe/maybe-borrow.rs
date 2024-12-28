Behaves similarly to [`maybe_borrow!`], but allows use of the `?` operator.
The return type must be one of the following:

- `Option<T>`
- `Result<T, E>`
- `ControlFlow<B, C>`
- `Poll<Result<T, E>>`
- `Poll<Option<Result<T, E>>>`

## Control flow

This macro internally places `$block` inside a closure, so returning or breaking from within will not work as expected.

* The <dfn>[`return_borrowed!`]</dfn> macro is used to return from the function that contains the `try_maybe_borrow!` invocation.
* If `return_borrowed!` is not used and the block exits normally, it will evaluate to the trailing expression like any block. The trailing expression cannot reference `$ptr`.
* If the macro completes without returning, the variable referenced by `$ptr` is still fully accessible because no borrowed data was allowed to escape the block.

## Examples

```rust
use std::{borrow::Borrow, collections::HashMap};
use maybe_borrow::try_maybe_borrow;

/// Finds the first key in `keys` that can be found in `map` and returns a mutable reference to its
/// value, or `Ok(None)` if none of the keys exist in `map`.
/// Returns Err(E) if an entry is found that contains an error.
fn get_first_available_mut<T, E: Clone>(
    mut map: &mut HashMap<String, Result<T, E>>,
    keys: impl IntoIterator<Item: Borrow<str>>,
) -> Result<Option<&mut T>, E> {
    for key in keys {
        try_maybe_borrow!(for<'x> |map| -> Result<Option<&'x mut T>, E> {
            if let Some(value) = map.get_mut(key.borrow()) {
                let value = value
                    .as_mut()
                    .map_err(|e| e.clone())?;

                // Use `return_borrowed!()` to return with borrowed data.
                return_borrowed!(Ok(Some(value)));
            }
        });
    }

    Ok(None)
}
```
