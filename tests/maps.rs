use maybe_borrow::maybe_borrow;
use std::{borrow::Borrow, collections::HashMap};

/// Finds the first key in `keys` that can be found in `map` and returns a mutable reference to its
/// value, or `None` if none of the keys exist in `map`.
pub fn get_first_available_mut<T>(
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
